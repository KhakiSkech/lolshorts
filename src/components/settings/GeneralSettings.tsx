import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";

interface GeneralSettingsProps {
  settings: {
    auto_start_with_league: boolean;
    minimize_to_tray: boolean;
    show_notifications: boolean;
    show_replay_popup: boolean;
  };
  onChange: (settings: any) => void;
}

export function GeneralSettings({ settings, onChange }: GeneralSettingsProps) {
  const handleChange = (key: string, value: boolean) => {
    onChange({ ...settings, [key]: value });
  };

  return (
    <Card>
      <CardContent className="space-y-6 pt-6">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="auto-start">Auto-start with League</Label>
            <p className="text-sm text-muted-foreground">
              Automatically start LoLShorts when League of Legends launches
            </p>
          </div>
          <Switch
            id="auto-start"
            checked={settings.auto_start_with_league}
            onCheckedChange={(checked) => handleChange("auto_start_with_league", checked)}
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="minimize-tray">Minimize to Tray</Label>
            <p className="text-sm text-muted-foreground">
              Keep running in background when window is closed
            </p>
          </div>
          <Switch
            id="minimize-tray"
            checked={settings.minimize_to_tray}
            onCheckedChange={(checked) => handleChange("minimize_to_tray", checked)}
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="notifications">Show Notifications</Label>
            <p className="text-sm text-muted-foreground">
              Show desktop notifications for events and recording status
            </p>
          </div>
          <Switch
            id="notifications"
            checked={settings.show_notifications}
            onCheckedChange={(checked) => handleChange("show_notifications", checked)}
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="replay-popup">Show Replay Detection Popup</Label>
            <p className="text-sm text-muted-foreground">
              Ask who to record when a replay is detected
            </p>
          </div>
          <Switch
            id="replay-popup"
            checked={settings.show_replay_popup}
            onCheckedChange={(checked) => handleChange("show_replay_popup", checked)}
          />
        </div>
      </CardContent>
    </Card>
  );
}
