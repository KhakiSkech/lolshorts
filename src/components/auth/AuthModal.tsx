import { useState } from "react";
import { LoginForm } from "./LoginForm";
import { SignupForm } from "./SignupForm";

interface AuthModalProps {
  onClose: () => void;
}

export function AuthModal({ onClose }: AuthModalProps) {
  const [mode, setMode] = useState<"login" | "signup">("login");

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="relative">
        <button
          onClick={onClose}
          className="absolute -top-4 -right-4 z-10 w-8 h-8 rounded-full bg-background border border-border flex items-center justify-center hover:bg-accent"
        >
          ✕
        </button>

        {mode === "login" ? (
          <LoginForm
            onSwitchToSignup={() => setMode("signup")}
            onSuccess={onClose}
          />
        ) : (
          <SignupForm
            onSwitchToLogin={() => setMode("login")}
            onSuccess={onClose}
          />
        )}
      </div>
    </div>
  );
}
