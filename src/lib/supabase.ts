import { createClient, Session, User } from '@supabase/supabase-js';
import { Store } from '@tauri-apps/plugin-store';

// These should be configured via environment variables in production
const SUPABASE_URL = import.meta.env.VITE_SUPABASE_URL || 'https://your-project.supabase.co';
const SUPABASE_ANON_KEY = import.meta.env.VITE_SUPABASE_ANON_KEY || 'your-anon-key';

// Initialize Supabase client
export const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
  auth: {
    autoRefreshToken: true,
    persistSession: false, // We'll handle persistence with Tauri store
  },
});

// Tauri store for persisting auth session
let store: Store | null = null;

async function getStore(): Promise<Store> {
  if (!store) {
    store = await Store.load('auth.json');
  }
  return store;
}

// Save session to Tauri store
export async function saveSession(session: Session | null): Promise<void> {
  const s = await getStore();
  if (session) {
    await s.set('session', {
      access_token: session.access_token,
      refresh_token: session.refresh_token,
      expires_at: session.expires_at,
      user: session.user,
    });
  } else {
    await s.delete('session');
  }
  await s.save();
}

// Load session from Tauri store
export async function loadSession(): Promise<Session | null> {
  try {
    const s = await getStore();
    const data = await s.get<{
      access_token: string;
      refresh_token: string;
      expires_at?: number;
      user: User;
    }>('session');

    if (data) {
      return {
        access_token: data.access_token,
        refresh_token: data.refresh_token,
        expires_at: data.expires_at,
        expires_in: data.expires_at ? data.expires_at - Math.floor(Date.now() / 1000) : 0,
        token_type: 'bearer',
        user: data.user,
      };
    }
    return null;
  } catch {
    return null;
  }
}

// Get current session
export async function getSession(): Promise<Session | null> {
  // First try to get from Supabase client
  const { data: { session } } = await supabase.auth.getSession();
  if (session) {
    return session;
  }

  // Try to restore from store
  const storedSession = await loadSession();
  if (storedSession) {
    // Try to refresh the session
    const { data, error } = await supabase.auth.setSession({
      access_token: storedSession.access_token,
      refresh_token: storedSession.refresh_token,
    });

    if (!error && data.session) {
      await saveSession(data.session);
      return data.session;
    }
  }

  return null;
}

// Sign in with magic link
export async function signInWithMagicLink(email: string): Promise<void> {
  const { error } = await supabase.auth.signInWithOtp({
    email,
    options: {
      // In a desktop app, we'd need to handle deep linking
      // For now, user will need to copy the magic link
      shouldCreateUser: true,
    },
  });

  if (error) {
    throw new Error(error.message);
  }
}

// Sign out
export async function signOut(): Promise<void> {
  await supabase.auth.signOut();
  await saveSession(null);
}

// Check if user has Pro status
export async function checkProStatus(userId: string): Promise<boolean> {
  try {
    const { data, error } = await supabase
      .from('users')
      .select('is_pro')
      .eq('id', userId)
      .single();

    if (error) {
      console.error('Error checking pro status:', error);
      return false;
    }

    return data?.is_pro || false;
  } catch {
    return false;
  }
}

// Listen for auth state changes
export function onAuthStateChange(
  callback: (event: string, session: Session | null) => void
): () => void {
  const { data: { subscription } } = supabase.auth.onAuthStateChange(
    async (event, session) => {
      await saveSession(session);
      callback(event, session);
    }
  );

  return () => subscription.unsubscribe();
}

// Verify magic link token (for deep link handling)
export async function verifyOtp(
  email: string,
  token: string
): Promise<Session | null> {
  const { data, error } = await supabase.auth.verifyOtp({
    email,
    token,
    type: 'magiclink',
  });

  if (error) {
    throw new Error(error.message);
  }

  if (data.session) {
    await saveSession(data.session);
  }

  return data.session;
}
