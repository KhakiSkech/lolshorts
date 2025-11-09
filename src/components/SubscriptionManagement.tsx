import { useState } from "react";
import { useTranslation } from 'react-i18next';
import { invoke } from "@tauri-apps/api/core";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  CreditCard,
  AlertCircle,
  CheckCircle2,
  XCircle,
  Calendar,
  DollarSign,
  Loader2
} from "lucide-react";

interface SubscriptionManagementProps {
  isOpen: boolean;
  onClose: () => void;
  currentTier: "FREE" | "PRO";
  expiresAt: string | null;
}

interface SubscriptionDetails {
  subscription_id: string;
  tier: "PRO";
  period: "MONTHLY" | "YEARLY";
  amount: number;
  status: "active" | "cancelled" | "expired";
  next_billing_date: string | null;
  created_at: string;
}

export function SubscriptionManagement({
  isOpen,
  onClose,
  currentTier,
  expiresAt
}: SubscriptionManagementProps) {
  const { t } = useTranslation();
  const [subscription, setSubscription] = useState<SubscriptionDetails | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);
  const [isCancelling, setIsCancelling] = useState(false);

  // Load subscription details when dialog opens
  useState(() => {
    if (isOpen && currentTier === "PRO") {
      loadSubscriptionDetails();
    }
  });

  const loadSubscriptionDetails = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const details = await invoke<SubscriptionDetails>("get_subscription_details");
      setSubscription(details);
    } catch (err) {
      console.error("Failed to load subscription:", err);
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancelSubscription = async () => {
    setIsCancelling(true);
    setError(null);

    try {
      await invoke("cancel_subscription");

      // Refresh subscription details
      await loadSubscriptionDetails();

      setShowCancelConfirm(false);

      // Show success message
      alert(t('settings.account.cancelSuccess'));
    } catch (err) {
      console.error("Failed to cancel subscription:", err);
      setError(err as string);
    } finally {
      setIsCancelling(false);
    }
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleDateString("ko-KR", {
      year: "numeric",
      month: "long",
      day: "numeric"
    });
  };

  const formatAmount = (amount: number): string => {
    return `₩${amount.toLocaleString("ko-KR")}`;
  };

  const getPeriodLabel = (period: "MONTHLY" | "YEARLY"): string => {
    return period === "MONTHLY" ? "Monthly" : "Yearly";
  };

  if (currentTier === "FREE") {
    return (
      <Dialog open={isOpen} onOpenChange={onClose}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>No Active Subscription</DialogTitle>
            <DialogDescription>
              You are currently on the FREE plan
            </DialogDescription>
          </DialogHeader>

          <div className="text-center py-6">
            <XCircle className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
            <p className="text-sm text-muted-foreground">
              Upgrade to PRO to access premium features and manage your subscription.
            </p>
          </div>

          <DialogFooter>
            <Button onClick={onClose}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <>
      <Dialog open={isOpen && !showCancelConfirm} onOpenChange={onClose}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <CreditCard className="w-5 h-5" />
              Subscription Management
            </DialogTitle>
            <DialogDescription>
              Manage your PRO subscription and billing details
            </DialogDescription>
          </DialogHeader>

          {error && (
            <Alert variant="destructive">
              <AlertCircle className="w-4 h-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-8 h-8 animate-spin text-primary" />
              <span className="ml-2 text-sm text-muted-foreground">Loading subscription details...</span>
            </div>
          ) : subscription ? (
            <div className="space-y-4">
              {/* Subscription Status */}
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Status</span>
                <Badge variant={subscription.status === "active" ? "default" : "destructive"}>
                  {subscription.status === "active" && <CheckCircle2 className="w-3 h-3 mr-1" />}
                  {subscription.status === "cancelled" && <XCircle className="w-3 h-3 mr-1" />}
                  {subscription.status.toUpperCase()}
                </Badge>
              </div>

              <Separator />

              {/* Plan Details */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Plan</span>
                  <span className="font-medium">PRO - {getPeriodLabel(subscription.period)}</span>
                </div>

                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground flex items-center gap-1">
                    <DollarSign className="w-4 h-4" />
                    Amount
                  </span>
                  <span className="font-medium">{formatAmount(subscription.amount)}</span>
                </div>

                {subscription.next_billing_date && subscription.status === "active" && (
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground flex items-center gap-1">
                      <Calendar className="w-4 h-4" />
                      Next Billing Date
                    </span>
                    <span className="font-medium">{formatDate(subscription.next_billing_date)}</span>
                  </div>
                )}

                {expiresAt && subscription.status === "cancelled" && (
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground flex items-center gap-1">
                      <Calendar className="w-4 h-4" />
                      Access Until
                    </span>
                    <span className="font-medium">{formatDate(expiresAt)}</span>
                  </div>
                )}

                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">Subscribed Since</span>
                  <span className="font-medium">{formatDate(subscription.created_at)}</span>
                </div>
              </div>

              <Separator />

              {/* Cancellation Info */}
              {subscription.status === "cancelled" ? (
                <Alert>
                  <AlertCircle className="w-4 h-4" />
                  <AlertDescription>
                    Your subscription has been cancelled. You will retain PRO access until {expiresAt ? formatDate(expiresAt) : "the end of your billing period"}.
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="text-sm text-muted-foreground space-y-2">
                  <p>• Auto-renews on {subscription.next_billing_date ? formatDate(subscription.next_billing_date) : "next billing date"}</p>
                  <p>• Cancel anytime - keep access until end of billing period</p>
                  <p>• Payment method: {subscription.period === "MONTHLY" ? "Monthly" : "Annual"} billing</p>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center py-6">
              <AlertCircle className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
              <p className="text-sm text-muted-foreground">
                Failed to load subscription details. Please try again.
              </p>
            </div>
          )}

          <DialogFooter className="flex gap-2">
            <Button variant="outline" onClick={onClose}>
              Close
            </Button>

            {subscription && subscription.status === "active" && (
              <Button
                variant="destructive"
                onClick={() => setShowCancelConfirm(true)}
              >
                Cancel Subscription
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Cancel Confirmation Dialog */}
      <Dialog open={showCancelConfirm} onOpenChange={setShowCancelConfirm}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-destructive">Cancel Subscription?</DialogTitle>
            <DialogDescription>
              Are you sure you want to cancel your PRO subscription?
            </DialogDescription>
          </DialogHeader>

          <Alert>
            <AlertCircle className="w-4 h-4" />
            <AlertDescription>
              You will retain PRO access until {subscription?.next_billing_date ? formatDate(subscription.next_billing_date) : "the end of your billing period"}. After that, your account will be downgraded to FREE.
            </AlertDescription>
          </Alert>

          <div className="space-y-2 text-sm">
            <p className="font-semibold">You will lose access to:</p>
            <ul className="list-disc list-inside text-muted-foreground space-y-1">
              <li>Unlimited clips per game</li>
              <li>Advanced video editor features</li>
              <li>1080p60 export</li>
              <li>Priority support</li>
              <li>No watermarks</li>
            </ul>
          </div>

          <DialogFooter className="flex gap-2">
            <Button
              variant="outline"
              onClick={() => setShowCancelConfirm(false)}
              disabled={isCancelling}
            >
              Keep Subscription
            </Button>

            <Button
              variant="destructive"
              onClick={handleCancelSubscription}
              disabled={isCancelling}
            >
              {isCancelling ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Cancelling...
                </>
              ) : (
                "Yes, Cancel Subscription"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
