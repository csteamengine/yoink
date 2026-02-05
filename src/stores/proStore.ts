import { create } from 'zustand';

export interface User {
  id: string;
  email: string;
  is_pro: boolean;
}

interface ProState {
  user: User | null;
  isAuthenticated: boolean;
  isPro: boolean;
  isLoading: boolean;
  error: string | null;
  lastValidation: number | null;

  // Actions
  checkAuth: () => Promise<void>;
  login: (email: string) => Promise<void>;
  logout: () => Promise<void>;
  validateProStatus: () => Promise<void>;
  openUpgrade: () => Promise<void>;
}

const VALIDATION_INTERVAL = 24 * 60 * 60 * 1000; // 24 hours

export const useProStore = create<ProState>((set, get) => ({
  user: null,
  isAuthenticated: false,
  isPro: false,
  isLoading: false,
  error: null,
  lastValidation: null,

  checkAuth: async () => {
    set({ isLoading: true, error: null });
    try {
      // Check stored session via Supabase
      // This will be implemented in lib/supabase.ts
      const { getSession } = await import('@/lib/supabase');
      const session = await getSession();

      if (session?.user) {
        const user: User = {
          id: session.user.id,
          email: session.user.email || '',
          is_pro: false, // Will be validated separately
        };
        set({ user, isAuthenticated: true });

        // Validate pro status
        await get().validateProStatus();
      } else {
        set({ user: null, isAuthenticated: false, isPro: false });
      }
    } catch (error) {
      set({ error: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  login: async (email: string) => {
    set({ isLoading: true, error: null });
    try {
      const { signInWithMagicLink } = await import('@/lib/supabase');
      await signInWithMagicLink(email);
      // User will receive magic link email
      // Auth state will be updated when they click the link
    } catch (error) {
      set({ error: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  logout: async () => {
    set({ isLoading: true, error: null });
    try {
      const { signOut } = await import('@/lib/supabase');
      await signOut();
      set({ user: null, isAuthenticated: false, isPro: false, lastValidation: null });
    } catch (error) {
      set({ error: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  validateProStatus: async () => {
    const { user, lastValidation } = get();

    // Skip if not authenticated
    if (!user) return;

    // Skip if validated recently
    if (lastValidation && Date.now() - lastValidation < VALIDATION_INTERVAL) {
      return;
    }

    try {
      const { checkProStatus } = await import('@/lib/supabase');
      const isPro = await checkProStatus(user.id);
      set({
        isPro,
        user: { ...user, is_pro: isPro },
        lastValidation: Date.now(),
      });
    } catch (error) {
      console.error('Failed to validate pro status:', error);
    }
  },

  openUpgrade: async () => {
    try {
      const { openCheckout } = await import('@/lib/stripe');
      const { user } = get();
      await openCheckout(user?.email);
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
