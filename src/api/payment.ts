import { cmd } from './client';

export interface SubscriptionDetails {
  is_active: boolean;
  tier: string;
  expires_at: string | null;
  auto_renew: boolean;
  payment_method: string | null;
}

export const paymentApi = {
  confirmPayment: (paymentKey: string, orderId: string, amount: number) => 
    cmd<void>('confirm_payment', { paymentKey, orderId, amount }),
    
  getSubscriptionDetails: () => 
    cmd<SubscriptionDetails>('get_subscription_details'),
    
  cancelSubscription: () => 
    cmd<void>('cancel_subscription'),
};
