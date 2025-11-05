import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useAuthStore } from "@/lib/auth";
import { AuthModal } from "@/components/auth";
import { PaymentModal } from "@/components/PaymentModal";
import { SubscriptionManagement } from "@/components/SubscriptionManagement";
import { EventFilterSettings } from "@/components/settings/EventFilterSettings";
import { GameModeSettings } from "@/components/settings/GameModeSettings";
import { VideoSettings } from "@/components/settings/VideoSettings";
import { AudioSettings } from "@/components/settings/AudioSettings";
import { ClipTimingSettings } from "@/components/settings/ClipTimingSettings";
import { HotkeySettings } from "@/components/settings/HotkeySettings";
import {
  Settings as SettingsIcon,
  CreditCard,
  Crown,
  Shield,
  CheckCircle2,
  XCircle,
  Save
} from "lucide-react";

interface LicenseInfo {
  tier: "FREE" | "PRO";
  expires_at: string | null;
  is_active: boolean;
}

interface RecordingSettings {
  event_filter: any;
  game_mode: any;
  video: any;
  audio: any;
  clip_timing: any;
  hotkeys: any;
  auto_start_with_league: boolean;
  minimize_to_tray: boolean;
  show_notifications: boolean;
}

export function Settings() {
  const { user, isAuthenticated } = useAuthStore();
  const [showAuthModal, setShowAuthModal] = useState(false);
  const [showPaymentModal, setShowPaymentModal] = useState(false);
  const [showSubscriptionManagement, setShowSubscriptionManagement] = useState(false);
  const [license, setLicense] = useState<LicenseInfo | null>(null);
  const [isLoadingLicense, setIsLoadingLicense] = useState(false);

  // Recording settings state
  const [recordingSettings, setRecordingSettings] = useState<RecordingSettings | null>(null);
  const [isLoadingSettings, setIsLoadingSettings] = useState(false);
  const [isSavingSettings, setIsSavingSettings] = useState(false);

  useEffect(() => {
    if (isAuthenticated && user) {
      loadLicenseInfo();
    }
  }, [isAuthenticated, user]);

  useEffect(() => {
    loadRecordingSettings();
  }, []);

  const loadLicenseInfo = async () => {
    setIsLoadingLicense(true);
    try {
      const licenseData = await invoke<LicenseInfo>("get_user_license");
      setLicense(licenseData);
    } catch (error) {
      console.error("Failed to load license info:", error);
    } finally {
      setIsLoadingLicense(false);
    }
  };

  const loadRecordingSettings = async () => {
    setIsLoadingSettings(true);
    try {
      const settings = await invoke<RecordingSettings>("get_recording_settings");
      setRecordingSettings(settings);
    } catch (error) {
      console.error("Failed to load recording settings:", error);
    } finally {
      setIsLoadingSettings(false);
    }
  };

  const saveRecordingSettings = async (settings: RecordingSettings) => {
    setIsSavingSettings(true);
    try {
      await invoke("save_recording_settings", { settings });
      setRecordingSettings(settings);
    } catch (error) {
      console.error("Failed to save recording settings:", error);
    } finally {
      setIsSavingSettings(false);
    }
  };

  const resetSettingsToDefault = async () => {
    setIsSavingSettings(true);
    try {
      const defaultSettings = await invoke<RecordingSettings>("reset_settings_to_default");
      setRecordingSettings(defaultSettings);
    } catch (error) {
      console.error("Failed to reset settings:", error);
    } finally {
      setIsSavingSettings(false);
    }
  };

  const handleUpgradeToPro = () => {
    if (!isAuthenticated) {
      setShowAuthModal(true);
      return;
    }

    setShowPaymentModal(true);
  };

  const handleManageSubscription = async () => {
    if (!isAuthenticated) {
      setShowAuthModal(true);
      return;
    }

    setShowSubscriptionManagement(true);
  };

  const formatExpirationDate = (dateStr: string | null): string => {
    if (!dateStr) return "Never";
    const date = new Date(dateStr);
    return date.toLocaleDateString("ko-KR", {
      year: "numeric",
      month: "long",
      day: "numeric"
    });
  };

  const getDaysRemaining = (dateStr: string | null): number => {
    if (!dateStr) return -1;
    const expirationDate = new Date(dateStr);
    const now = new Date();
    const diff = expirationDate.getTime() - now.getTime();
    return Math.ceil(diff / (1000 * 60 * 60 * 24));
  };

  return (
    <div>
      <h2 className="text-3xl font-bold mb-6">Settings</h2>

      <div className="space-y-6">
        {/* License & Subscription */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Crown className="w-6 h-6" />
              License & Subscription
            </CardTitle>
            <CardDescription>
              Your current plan and subscription details
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {!isAuthenticated ? (
              <div className="text-center py-8">
                <Shield className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                <p className="text-lg font-semibold mb-2">Login Required</p>
                <p className="text-sm text-muted-foreground mb-4">
                  Please login to view and manage your subscription
                </p>
                <Button onClick={() => setShowAuthModal(true)}>
                  Login or Sign Up
                </Button>
              </div>
            ) : isLoadingLicense ? (
              <div className="text-center py-8">
                <p className="text-sm text-muted-foreground">Loading license information...</p>
              </div>
            ) : license ? (
              <>
                {/* Current Plan */}
                <div>
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <h3 className="text-lg font-semibold flex items-center gap-2">
                        Current Plan
                        <Badge variant={license.tier === "PRO" ? "default" : "secondary"} className="text-base">
                          {license.tier}
                        </Badge>
                      </h3>
                      <p className="text-sm text-muted-foreground mt-1">
                        {license.tier === "PRO"
                          ? "Unlimited clips, priority support, and advanced features"
                          : "Free tier with basic recording features"
                        }
                      </p>
                    </div>
                    {license.tier === "FREE" && (
                      <Button onClick={handleUpgradeToPro}>
                        <Crown className="w-4 h-4 mr-2" />
                        Upgrade to PRO
                      </Button>
                    )}
                  </div>

                  <Separator className="my-4" />

                  {/* Plan Details */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <p className="text-sm text-muted-foreground">Status</p>
                      <div className="flex items-center gap-2 mt-1">
                        {license.is_active ? (
                          <>
                            <CheckCircle2 className="w-4 h-4 text-green-500" />
                            <span className="font-medium">Active</span>
                          </>
                        ) : (
                          <>
                            <XCircle className="w-4 h-4 text-destructive" />
                            <span className="font-medium">Inactive</span>
                          </>
                        )}
                      </div>
                    </div>

                    {license.tier === "PRO" && license.expires_at && (
                      <>
                        <div>
                          <p className="text-sm text-muted-foreground">Expires On</p>
                          <p className="font-medium mt-1">{formatExpirationDate(license.expires_at)}</p>
                        </div>

                        {getDaysRemaining(license.expires_at) > 0 && (
                          <div>
                            <p className="text-sm text-muted-foreground">Days Remaining</p>
                            <p className="font-medium mt-1">
                              {getDaysRemaining(license.expires_at)} days
                            </p>
                          </div>
                        )}
                      </>
                    )}

                    <div>
                      <p className="text-sm text-muted-foreground">Account Email</p>
                      <p className="font-medium mt-1">{user?.email || "N/A"}</p>
                    </div>
                  </div>

                  {license.tier === "PRO" && (
                    <div className="mt-4">
                      <Button onClick={handleManageSubscription} variant="outline">
                        <CreditCard className="w-4 h-4 mr-2" />
                        Manage Subscription
                      </Button>
                    </div>
                  )}
                </div>

                {/* Plan Comparison */}
                {license.tier === "FREE" && (
                  <>
                    <Separator />
                    <div>
                      <h3 className="text-lg font-semibold mb-3">Why Upgrade to PRO?</h3>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <div className="flex items-start gap-2">
                          <CheckCircle2 className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                          <div>
                            <p className="font-medium text-sm">Unlimited Clips</p>
                            <p className="text-xs text-muted-foreground">Record as many highlights as you want</p>
                          </div>
                        </div>
                        <div className="flex items-start gap-2">
                          <CheckCircle2 className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                          <div>
                            <p className="font-medium text-sm">Advanced Editor</p>
                            <p className="text-xs text-muted-foreground">Premium transitions and effects</p>
                          </div>
                        </div>
                        <div className="flex items-start gap-2">
                          <CheckCircle2 className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                          <div>
                            <p className="font-medium text-sm">Priority Support</p>
                            <p className="text-xs text-muted-foreground">Get help faster</p>
                          </div>
                        </div>
                        <div className="flex items-start gap-2">
                          <CheckCircle2 className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                          <div>
                            <p className="font-medium text-sm">No Watermarks</p>
                            <p className="text-xs text-muted-foreground">Export clean videos</p>
                          </div>
                        </div>
                      </div>
                      <div className="mt-4 p-4 bg-primary/10 rounded-lg">
                        <p className="text-sm">
                          <strong>PRO Pricing:</strong> ₩9,900/month or ₩99,000/year (Save 17%)
                        </p>
                      </div>
                    </div>
                  </>
                )}
              </>
            ) : (
              <div className="text-center py-8">
                <p className="text-sm text-muted-foreground">Failed to load license information</p>
                <Button onClick={loadLicenseInfo} variant="outline" className="mt-4">
                  Retry
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Recording Configuration */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <SettingsIcon className="w-6 h-6" />
                  Recording Configuration
                </CardTitle>
                <CardDescription>
                  Configure automatic clip recording preferences
                </CardDescription>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={resetSettingsToDefault}
                disabled={isSavingSettings || !recordingSettings}
              >
                Reset to Defaults
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {isLoadingSettings ? (
              <div className="text-center py-8">
                <p className="text-sm text-muted-foreground">Loading settings...</p>
              </div>
            ) : recordingSettings ? (
              <Tabs defaultValue="events" className="w-full">
                <TabsList className="grid w-full grid-cols-6">
                  <TabsTrigger value="events">Events</TabsTrigger>
                  <TabsTrigger value="modes">Game Modes</TabsTrigger>
                  <TabsTrigger value="video">Video</TabsTrigger>
                  <TabsTrigger value="audio">Audio</TabsTrigger>
                  <TabsTrigger value="timing">Timing</TabsTrigger>
                  <TabsTrigger value="hotkeys">Hotkeys</TabsTrigger>
                </TabsList>

                <div className="mt-6">
                  <TabsContent value="events" className="space-y-4">
                    <EventFilterSettings
                      settings={recordingSettings.event_filter}
                      onChange={(eventFilter) => {
                        const updated = { ...recordingSettings, event_filter: eventFilter };
                        saveRecordingSettings(updated);
                      }}
                    />
                  </TabsContent>

                  <TabsContent value="modes" className="space-y-4">
                    <GameModeSettings
                      settings={recordingSettings.game_mode}
                      onChange={(gameMode) => {
                        const updated = { ...recordingSettings, game_mode: gameMode };
                        saveRecordingSettings(updated);
                      }}
                    />
                  </TabsContent>

                  <TabsContent value="video" className="space-y-4">
                    <VideoSettings
                      settings={recordingSettings.video}
                      onChange={(video) => {
                        const updated = { ...recordingSettings, video };
                        saveRecordingSettings(updated);
                      }}
                    />
                  </TabsContent>

                  <TabsContent value="audio" className="space-y-4">
                    <AudioSettings
                      settings={recordingSettings.audio}
                      onChange={(audio) => {
                        const updated = { ...recordingSettings, audio };
                        saveRecordingSettings(updated);
                      }}
                    />
                  </TabsContent>

                  <TabsContent value="timing" className="space-y-4">
                    <ClipTimingSettings
                      settings={recordingSettings.clip_timing}
                      onChange={(clipTiming) => {
                        const updated = { ...recordingSettings, clip_timing: clipTiming };
                        saveRecordingSettings(updated);
                      }}
                    />
                  </TabsContent>

                  <TabsContent value="hotkeys" className="space-y-4">
                    <HotkeySettings
                      settings={recordingSettings.hotkeys}
                      onChange={(hotkeys) => {
                        const updated = { ...recordingSettings, hotkeys };
                        saveRecordingSettings(updated);
                      }}
                    />
                  </TabsContent>
                </div>

                {isSavingSettings && (
                  <div className="flex items-center justify-center gap-2 mt-4 text-sm text-muted-foreground">
                    <Save className="w-4 h-4 animate-pulse" />
                    Saving...
                  </div>
                )}
              </Tabs>
            ) : (
              <div className="text-center py-8">
                <p className="text-sm text-muted-foreground">Failed to load settings</p>
                <Button onClick={loadRecordingSettings} variant="outline" className="mt-4">
                  Retry
                </Button>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Account Information */}
        {isAuthenticated && user && (
          <Card>
            <CardHeader>
              <CardTitle>Account Information</CardTitle>
              <CardDescription>Your account details</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-sm text-muted-foreground">Email</span>
                  <span className="text-sm font-medium">{user.email}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-muted-foreground">User ID</span>
                  <span className="text-sm font-mono">{user.id.substring(0, 8)}...</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-muted-foreground">License Tier</span>
                  <Badge variant={user.tier === "Pro" ? "default" : "secondary"}>
                    {user.tier}
                  </Badge>
                </div>
              </div>
            </CardContent>
          </Card>
        )}
      </div>

      {/* Auth Modal */}
      {showAuthModal && <AuthModal onClose={() => setShowAuthModal(false)} />}

      {/* Payment Modal */}
      <PaymentModal
        isOpen={showPaymentModal}
        onClose={() => {
          setShowPaymentModal(false);
          // Reload license info after closing payment modal (in case payment was completed)
          if (isAuthenticated) {
            loadLicenseInfo();
          }
        }}
      />

      {/* Subscription Management Modal */}
      <SubscriptionManagement
        isOpen={showSubscriptionManagement}
        onClose={() => {
          setShowSubscriptionManagement(false);
          // Reload license info after closing (in case subscription was cancelled)
          if (isAuthenticated) {
            loadLicenseInfo();
          }
        }}
        currentTier={license?.tier || "FREE"}
        expiresAt={license?.expires_at || null}
      />
    </div>
  );
}
