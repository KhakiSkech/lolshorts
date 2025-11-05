import { useEffect, useState } from "react";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { CheckCircle2, AlertCircle, Loader2 } from "lucide-react";
import { useAuthStore } from "@/lib/auth";

export function PaymentSuccess() {
  const searchParams = useSearch({ from: "/payment/success" }) as Record<string, string>;
  const navigate = useNavigate();
  const { checkAuth } = useAuthStore();

  const [status, setStatus] = useState<"processing" | "success" | "error">("processing");
  const [errorMessage, setErrorMessage] = useState<string>("");

  useEffect(() => {
    confirmPayment();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const confirmPayment = async () => {
    try {
      // Get payment details from URL params
      const paymentKey = searchParams.paymentKey;
      const orderId = searchParams.orderId;
      const amount = searchParams.amount;

      if (!paymentKey || !orderId || !amount) {
        throw new Error("Missing payment information in URL");
      }

      // Verify with stored order info
      const pendingOrderId = sessionStorage.getItem("pending_order_id");
      const pendingAmount = sessionStorage.getItem("pending_amount");

      if (orderId !== pendingOrderId) {
        throw new Error("Order ID mismatch");
      }

      if (amount !== pendingAmount) {
        throw new Error("Payment amount mismatch");
      }

      // Confirm payment with backend
      await invoke("confirm_payment", {
        paymentKey,
        orderId,
        amount: parseInt(amount)
      });

      // Clear stored order info
      sessionStorage.removeItem("pending_order_id");
      sessionStorage.removeItem("pending_amount");

      // Refresh auth to update license tier
      await checkAuth();

      setStatus("success");
    } catch (error) {
      console.error("Payment confirmation failed:", error);
      setErrorMessage(error as string);
      setStatus("error");
    }
  };

  if (status === "processing") {
    return (
      <div className="min-h-screen flex items-center justify-center p-6">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="text-center">Processing Payment</CardTitle>
            <CardDescription className="text-center">
              Please wait while we confirm your subscription
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col items-center space-y-4">
            <Loader2 className="w-16 h-16 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground text-center">
              Do not close this window
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (status === "error") {
    return (
      <div className="min-h-screen flex items-center justify-center p-6">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertCircle className="w-6 h-6" />
              Payment Confirmation Failed
            </CardTitle>
            <CardDescription>
              We couldn't verify your payment
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert variant="destructive">
              <AlertCircle className="w-4 h-4" />
              <AlertDescription>{errorMessage}</AlertDescription>
            </Alert>
            <p className="text-sm text-muted-foreground">
              If you were charged, please contact support with your order information.
              We'll resolve this as soon as possible.
            </p>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={() => navigate({ to: "/settings" })}
                className="flex-1"
              >
                Go to Settings
              </Button>
              <Button
                onClick={() => window.location.reload()}
                className="flex-1"
              >
                Try Again
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-green-600 dark:text-green-400">
            <CheckCircle2 className="w-6 h-6" />
            Payment Successful!
          </CardTitle>
          <CardDescription>
            Welcome to LoLShorts PRO
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
            <p className="font-semibold text-green-600 dark:text-green-400 mb-2">
              🎉 Subscription Activated
            </p>
            <p className="text-sm text-muted-foreground">
              Your PRO subscription is now active. Enjoy unlimited clips, advanced editing features, and priority support!
            </p>
          </div>

          <div className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Order ID:</span>
              <span className="font-mono text-xs">{searchParams.orderId}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Amount Paid:</span>
              <span className="font-medium">
                ₩{parseInt(searchParams.amount || "0").toLocaleString("ko-KR")}
              </span>
            </div>
          </div>

          <div className="pt-4">
            <Button
              onClick={() => navigate({ to: "/settings" })}
              className="w-full"
            >
              Go to Settings
            </Button>
          </div>

          <p className="text-xs text-center text-muted-foreground">
            A confirmation email has been sent to your registered email address
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
