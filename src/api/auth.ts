import { cmd } from './client';

export interface User {
  id: string;
  email: string;
  tier: 'Free' | 'Pro';
  // Add other fields matching Rust struct
}

export interface LicenseInfo {
  tier: string;
  expires_at: string | null;
  is_active: boolean;
}

export interface SubscriptionDetails {
  is_active: boolean;
  tier: string;
  expires_at: string | null;
  auto_renew: boolean;
  payment_method: string | null;
  payment_available: boolean;
  payment_message: string | null;
}

export const authApi = {
  login: (email: string, password: string) =>
    cmd<User>('login', { email, password }),

  signup: (email: string, password: string) =>
    cmd<User>('signup', { email, password }),

  logout: () =>
    cmd<void>('logout'),

  getUserStatus: () =>
    cmd<User | null>('get_user_status'),

  getLicenseInfo: () =>
    cmd<LicenseInfo | null>('get_license_info'),

  getUserLicense: () =>
    cmd<LicenseInfo>('get_user_license'),

  refreshToken: () =>
    cmd<User>('refresh_token'),

  setSession: (accessToken: string, refreshToken: string, userId: string, email: string) =>
    cmd<void>('set_session', { accessToken, refreshToken, userId, email }),

  // Subscription/Payment APIs
  getSubscriptionDetails: () =>
    cmd<SubscriptionDetails>('get_subscription_details'),

  openPaymentPage: () =>
    cmd<string>('open_payment_page'),

  cancelSubscription: () =>
    cmd<void>('cancel_subscription'),
};

