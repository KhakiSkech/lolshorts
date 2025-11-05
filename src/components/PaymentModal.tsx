import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Crown, Check, AlertCircle } from "lucide-react";
import { open } from "@tauri-apps/plugin-shell";

interface PaymentModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface SubscriptionResponse {
  checkout_url: string;
  order_id: string;
}

export function PaymentModal({ isOpen, onClose }: PaymentModalProps) {
  const [period, setPeriod] = useState<"MONTHLY" | "YEARLY">("MONTHLY");
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubscribe = async () => {
    setIsProcessing(true);
    setError(null);

    try {
      const response = await invoke<SubscriptionResponse>("create_subscription", {
        request: { period }
      });

      // Open checkout URL in external browser
      await open(response.checkout_url);

      // Store order_id for later confirmation
      sessionStorage.setItem("pending_order_id", response.order_id);
      sessionStorage.setItem("pending_amount", period === "MONTHLY" ? "9900" : "99000");

      onClose();
    } catch (err) {
      console.error("Failed to create subscription:", err);
      setError(err as string);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Crown className="w-6 h-6 text-yellow-500" />
            Upgrade to LoLShorts PRO
          </DialogTitle>
          <DialogDescription>
            Choose your subscription plan and unlock unlimited features
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Plan Selection */}
          <RadioGroup value={period} onValueChange={(val) => setPeriod(val as "MONTHLY" | "YEARLY")}>
            <div className="space-y-3">
              {/* Monthly Plan */}
              <div
                className={`relative flex items-start p-4 border-2 rounded-lg cursor-pointer transition-colors ${
                  period === "MONTHLY"
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-primary/50"
                }`}
                onClick={() => setPeriod("MONTHLY")}
              >
                <RadioGroupItem value="MONTHLY" id="monthly" className="mt-1" />
                <Label htmlFor="monthly" className="flex-1 ml-3 cursor-pointer">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="font-semibold">Monthly Plan</p>
                      <p className="text-sm text-muted-foreground">₩9,900/month</p>
                    </div>
                    {period === "MONTHLY" && (
                      <Check className="w-5 h-5 text-primary" />
                    )}
                  </div>
                </Label>
              </div>

              {/* Yearly Plan */}
              <div
                className={`relative flex items-start p-4 border-2 rounded-lg cursor-pointer transition-colors ${
                  period === "YEARLY"
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-primary/50"
                }`}
                onClick={() => setPeriod("YEARLY")}
              >
                <RadioGroupItem value="YEARLY" id="yearly" className="mt-1" />
                <Label htmlFor="yearly" className="flex-1 ml-3 cursor-pointer">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="font-semibold flex items-center gap-2">
                        Yearly Plan
                        <span className="px-2 py-0.5 text-xs font-medium bg-green-500/10 text-green-600 dark:text-green-400 rounded">
                          Save 17%
                        </span>
                      </p>
                      <p className="text-sm text-muted-foreground">
                        ₩99,000/year
                        <span className="ml-2 text-xs line-through opacity-60">₩118,800</span>
                      </p>
                    </div>
                    {period === "YEARLY" && (
                      <Check className="w-5 h-5 text-primary" />
                    )}
                  </div>
                </Label>
              </div>
            </div>
          </RadioGroup>

          {/* Features List */}
          <div className="p-4 bg-muted/50 rounded-lg">
            <p className="font-semibold mb-3">PRO Features:</p>
            <div className="space-y-2 text-sm">
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4 text-green-500" />
                <span>Unlimited clips per game</span>
              </div>
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4 text-green-500" />
                <span>Advanced video editor with premium effects</span>
              </div>
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4 text-green-500" />
                <span>No watermarks on exported videos</span>
              </div>
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4 text-green-500" />
                <span>Priority support and updates</span>
              </div>
              <div className="flex items-center gap-2">
                <Check className="w-4 h-4 text-green-500" />
                <span>Cloud storage for your highlights</span>
              </div>
            </div>
          </div>

          {/* Error Display */}
          {error && (
            <Alert variant="destructive">
              <AlertCircle className="w-4 h-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {/* Action Buttons */}
          <div className="flex gap-3">
            <Button
              variant="outline"
              onClick={onClose}
              disabled={isProcessing}
              className="flex-1"
            >
              Cancel
            </Button>
            <Button
              onClick={handleSubscribe}
              disabled={isProcessing}
              className="flex-1"
            >
              {isProcessing ? "Processing..." : "Continue to Payment"}
            </Button>
          </div>

          <p className="text-xs text-center text-muted-foreground">
            You'll be redirected to Toss Payments to complete your purchase securely
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
}
