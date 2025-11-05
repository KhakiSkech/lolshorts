import { useNavigate, useSearch } from "@tanstack/react-router";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { XCircle, AlertCircle } from "lucide-react";

export function PaymentFail() {
  const searchParams = useSearch({ from: "/payment/fail" }) as Record<string, string>;
  const navigate = useNavigate();

  const errorCode = searchParams.code;
  const errorMessage = searchParams.message;

  const getErrorDetails = () => {
    switch (errorCode) {
      case "PAY_PROCESS_CANCELED":
        return {
          title: "Payment Cancelled",
          description: "You cancelled the payment process. No charges were made to your account."
        };
      case "PAY_PROCESS_ABORTED":
        return {
          title: "Payment Aborted",
          description: "The payment process was interrupted. Please try again."
        };
      case "REJECT_CARD_COMPANY":
        return {
          title: "Card Declined",
          description: "Your card was declined by the card company. Please try a different payment method."
        };
      default:
        return {
          title: "Payment Failed",
          description: errorMessage || "An error occurred during payment processing."
        };
    }
  };

  const errorDetails = getErrorDetails();

  const handleRetry = () => {
    // Clear any stored order info
    sessionStorage.removeItem("pending_order_id");
    sessionStorage.removeItem("pending_amount");

    // Go back to settings to retry
    navigate({ to: "/settings" });
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-6">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-destructive">
            <XCircle className="w-6 h-6" />
            {errorDetails.title}
          </CardTitle>
          <CardDescription>
            {errorDetails.description}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {errorMessage && (
            <Alert variant="destructive">
              <AlertCircle className="w-4 h-4" />
              <AlertDescription>{errorMessage}</AlertDescription>
            </Alert>
          )}

          {errorCode && (
            <div className="text-xs text-muted-foreground">
              <p>Error Code: {errorCode}</p>
            </div>
          )}

          <div className="p-4 bg-muted/50 rounded-lg">
            <p className="text-sm font-semibold mb-2">What can you do?</p>
            <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
              <li>Check your payment method and try again</li>
              <li>Try a different payment method</li>
              <li>Contact your card issuer if the problem persists</li>
              <li>Contact our support team if you need assistance</li>
            </ul>
          </div>

          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={() => navigate({ to: "/" })}
              className="flex-1"
            >
              Go Home
            </Button>
            <Button
              onClick={handleRetry}
              className="flex-1"
            >
              Try Again
            </Button>
          </div>

          <p className="text-xs text-center text-muted-foreground">
            No charges were made to your account
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
