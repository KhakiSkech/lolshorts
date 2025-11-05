import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Keyboard, RotateCcw } from "lucide-react";

interface HotkeySettings {
  manual_save_clip: string;
  toggle_recording: string;
  delete_last_clip: string;
}

interface HotkeySettingsProps {
  settings: HotkeySettings;
  onChange: (settings: HotkeySettings) => void;
}

export function HotkeySettings({ settings, onChange }: HotkeySettingsProps) {
  const [recording, setRecording] = useState<keyof HotkeySettings | null>(null);

  const updateHotkey = (key: keyof HotkeySettings, value: string) => {
    onChange({ ...settings, [key]: value });
  };

  const resetToDefaults = () => {
    onChange({
      manual_save_clip: "F8",
      toggle_recording: "F9",
      delete_last_clip: "F10",
    });
  };

  const startRecording = (key: keyof HotkeySettings) => {
    setRecording(key);
  };

  const handleKeyDown = (event: React.KeyboardEvent, key: keyof HotkeySettings) => {
    if (!recording || recording !== key) return;

    event.preventDefault();
    event.stopPropagation();

    let hotkey = "";

    // Build hotkey string
    if (event.ctrlKey) hotkey += "Ctrl+";
    if (event.altKey) hotkey += "Alt+";
    if (event.shiftKey) hotkey += "Shift+";

    // Add main key
    if (event.key === "Control" || event.key === "Alt" || event.key === "Shift") {
      return; // Don't capture modifier keys alone
    }

    if (event.key.length === 1) {
      hotkey += event.key.toUpperCase();
    } else {
      hotkey += event.key;
    }

    updateHotkey(key, hotkey);
    setRecording(null);
  };

  const getHotkeyDisplay = (key: keyof HotkeySettings): string => {
    if (recording === key) {
      return "Press any key...";
    }
    return settings[key];
  };

  return (
    <div className="space-y-6">
      {/* Info Card */}
      <Card className="bg-muted/50">
        <CardContent className="pt-6">
          <div className="flex items-start gap-3">
            <Keyboard className="w-5 h-5 text-muted-foreground mt-0.5" />
            <div className="space-y-1 text-sm">
              <p className="font-semibold">Global Hotkeys</p>
              <p className="text-muted-foreground">
                These hotkeys work even when League of Legends is in focus. Click on a hotkey field
                and press your desired key combination to change it.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Manual Save Clip */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Save Clip Manually</CardTitle>
          <CardDescription>
            Instantly save the last 10-30 seconds as a clip
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-3">
            <div className="flex-1">
              <Input
                value={getHotkeyDisplay("manual_save_clip")}
                onFocus={() => startRecording("manual_save_clip")}
                onBlur={() => setRecording(null)}
                onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => handleKeyDown(e, "manual_save_clip")}
                readOnly
                className={recording === "manual_save_clip" ? "border-primary" : ""}
                placeholder="Click to set hotkey"
              />
            </div>
            {settings.manual_save_clip !== "F8" && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => updateHotkey("manual_save_clip", "F8")}
              >
                <RotateCcw className="w-4 h-4" />
              </Button>
            )}
          </div>
          <p className="text-xs text-muted-foreground">
            Default: F8 (Press to restore default)
          </p>
        </CardContent>
      </Card>

      {/* Toggle Recording */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Toggle Recording</CardTitle>
          <CardDescription>
            Start or stop automatic recording
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-3">
            <div className="flex-1">
              <Input
                value={getHotkeyDisplay("toggle_recording")}
                onFocus={() => startRecording("toggle_recording")}
                onBlur={() => setRecording(null)}
                onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => handleKeyDown(e, "toggle_recording")}
                readOnly
                className={recording === "toggle_recording" ? "border-primary" : ""}
                placeholder="Click to set hotkey"
              />
            </div>
            {settings.toggle_recording !== "F9" && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => updateHotkey("toggle_recording", "F9")}
              >
                <RotateCcw className="w-4 h-4" />
              </Button>
            )}
          </div>
          <p className="text-xs text-muted-foreground">
            Default: F9 (Press to restore default)
          </p>
        </CardContent>
      </Card>

      {/* Delete Last Clip */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Delete Last Clip</CardTitle>
          <CardDescription>
            Remove the most recently saved clip
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-3">
            <div className="flex-1">
              <Input
                value={getHotkeyDisplay("delete_last_clip")}
                onFocus={() => startRecording("delete_last_clip")}
                onBlur={() => setRecording(null)}
                onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => handleKeyDown(e, "delete_last_clip")}
                readOnly
                className={recording === "delete_last_clip" ? "border-primary" : ""}
                placeholder="Click to set hotkey"
              />
            </div>
            {settings.delete_last_clip !== "F10" && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => updateHotkey("delete_last_clip", "F10")}
              >
                <RotateCcw className="w-4 h-4" />
              </Button>
            )}
          </div>
          <p className="text-xs text-muted-foreground">
            Default: F10 (Press to restore default)
          </p>
        </CardContent>
      </Card>

      {/* Reset All */}
      <div className="pt-4">
        <Button variant="outline" onClick={resetToDefaults}>
          <RotateCcw className="w-4 h-4 mr-2" />
          Reset All Hotkeys to Defaults
        </Button>
      </div>
    </div>
  );
}
