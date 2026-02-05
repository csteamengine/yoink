import { open } from '@tauri-apps/plugin-shell';

// Stripe configuration
// In production, these should be loaded from environment variables
const STRIPE_CHECKOUT_URL = import.meta.env.VITE_STRIPE_CHECKOUT_URL ||
  'https://checkout.stripe.com/pay/your-checkout-session';

const PRICE_ID = import.meta.env.VITE_STRIPE_PRICE_ID || 'price_your_price_id';

// Backend URL for creating checkout sessions
// In production, this would be a Supabase Edge Function or similar
const CHECKOUT_API_URL = import.meta.env.VITE_CHECKOUT_API_URL ||
  'https://your-project.supabase.co/functions/v1/create-checkout';

interface CheckoutSessionResponse {
  url: string;
  sessionId: string;
}

// Create a Stripe checkout session
export async function createCheckoutSession(
  email?: string
): Promise<CheckoutSessionResponse> {
  const response = await fetch(CHECKOUT_API_URL, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      priceId: PRICE_ID,
      email,
      successUrl: 'yoink://payment-success',
      cancelUrl: 'yoink://payment-cancel',
    }),
  });

  if (!response.ok) {
    throw new Error('Failed to create checkout session');
  }

  return response.json();
}

// Open Stripe checkout in system browser
export async function openCheckout(email?: string): Promise<void> {
  try {
    // Try to create a custom checkout session with prefilled email
    const { url } = await createCheckoutSession(email);
    await open(url);
  } catch (error) {
    // Fallback to static checkout URL
    console.warn('Failed to create checkout session, using fallback:', error);
    await open(STRIPE_CHECKOUT_URL);
  }
}

// Verify payment success (called after redirect from Stripe)
export async function verifyPayment(sessionId: string): Promise<boolean> {
  try {
    const response = await fetch(`${CHECKOUT_API_URL}/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ sessionId }),
    });

    if (!response.ok) {
      return false;
    }

    const data = await response.json();
    return data.paid === true;
  } catch {
    return false;
  }
}

// Format price for display
export function formatPrice(cents: number, currency = 'USD'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
  }).format(cents / 100);
}

// Price constants
export const PRO_PRICE = 499; // $4.99 in cents
export const PRO_PRICE_DISPLAY = formatPrice(PRO_PRICE);
