import { useState } from "react";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { LoginForm } from "./LoginForm";
import { SignupForm } from "./SignupForm";

interface AuthModalProps {
  open: boolean;
  onClose: () => void;
  defaultMode?: "login" | "signup";
}

export function AuthModal({ open, onClose, defaultMode = "login" }: AuthModalProps) {
  const [mode, setMode] = useState<"login" | "signup">(defaultMode);

  const handleClose = () => {
    // Reset to default mode when closing
    setMode(defaultMode);
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogTitle className="sr-only">
          {mode === "login" ? "Sign In" : "Create Account"}
        </DialogTitle>
        {mode === "login" ? (
          <LoginForm
            onSwitchToSignup={() => setMode("signup")}
            onSuccess={handleClose}
          />
        ) : (
          <SignupForm
            onSwitchToLogin={() => setMode("login")}
            onSuccess={handleClose}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}
