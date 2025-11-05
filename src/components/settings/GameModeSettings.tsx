import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface GameModeSettings {
  record_ranked_solo: boolean;
  record_ranked_flex: boolean;
  record_normal: boolean;
  record_quick_play: boolean;
  record_aram: boolean;
  record_arena: boolean;
  record_special: boolean;
  record_custom: boolean;
  record_practice: boolean;
}

interface GameModeSettingsProps {
  settings: GameModeSettings;
  onChange: (settings: GameModeSettings) => void;
}

export function GameModeSettings({ settings, onChange }: GameModeSettingsProps) {
  const updateSetting = (key: keyof GameModeSettings, value: boolean) => {
    onChange({ ...settings, [key]: value });
  };

  const applyPreset = (preset: "competitive" | "all" | "ranked-only") => {
    switch (preset) {
      case "competitive":
        onChange({
          record_ranked_solo: true,
          record_ranked_flex: true,
          record_normal: true,
          record_quick_play: true,
          record_aram: true,
          record_arena: true,
          record_special: false,
          record_custom: false,
          record_practice: false,
        });
        break;
      case "all":
        onChange({
          record_ranked_solo: true,
          record_ranked_flex: true,
          record_normal: true,
          record_quick_play: true,
          record_aram: true,
          record_arena: true,
          record_special: true,
          record_custom: true,
          record_practice: true,
        });
        break;
      case "ranked-only":
        onChange({
          record_ranked_solo: true,
          record_ranked_flex: true,
          record_normal: false,
          record_quick_play: false,
          record_aram: false,
          record_arena: false,
          record_special: false,
          record_custom: false,
          record_practice: false,
        });
        break;
    }
  };

  return (
    <div className="space-y-6">
      {/* Presets */}
      <div>
        <h3 className="text-sm font-semibold mb-3">Quick Presets</h3>
        <div className="flex gap-2 flex-wrap">
          <Button variant="outline" size="sm" onClick={() => applyPreset("competitive")}>
            Competitive Modes
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("ranked-only")}>
            Ranked Only
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("all")}>
            All Modes
          </Button>
        </div>
      </div>

      {/* Ranked Modes */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Ranked Modes</CardTitle>
          <CardDescription>
            Competitive ranked gameplay
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_ranked_solo" className="cursor-pointer">
                Ranked Solo/Duo
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                Competitive 5v5 solo/duo queue
              </p>
            </div>
            <Switch
              id="record_ranked_solo"
              checked={settings.record_ranked_solo}
              onCheckedChange={(checked: boolean) => updateSetting("record_ranked_solo", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_ranked_flex" className="cursor-pointer">
                Ranked Flex
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                Competitive 5v5 flex queue
              </p>
            </div>
            <Switch
              id="record_ranked_flex"
              checked={settings.record_ranked_flex}
              onCheckedChange={(checked: boolean) => updateSetting("record_ranked_flex", checked)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Normal Modes */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Normal Modes</CardTitle>
          <CardDescription>
            Casual gameplay modes
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_normal" className="cursor-pointer">
                Normal Draft/Blind
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                Classic 5v5 Summoner's Rift
              </p>
            </div>
            <Switch
              id="record_normal"
              checked={settings.record_normal}
              onCheckedChange={(checked: boolean) => updateSetting("record_normal", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_quick_play" className="cursor-pointer">
                Quick Play
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                Fast-paced casual matches
              </p>
            </div>
            <Switch
              id="record_quick_play"
              checked={settings.record_quick_play}
              onCheckedChange={(checked: boolean) => updateSetting("record_quick_play", checked)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Alternative Modes */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Alternative Modes</CardTitle>
          <CardDescription>
            ARAM, Arena, and special game modes
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_aram" className="cursor-pointer">
                ARAM
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                All Random All Mid on Howling Abyss
              </p>
            </div>
            <Switch
              id="record_aram"
              checked={settings.record_aram}
              onCheckedChange={(checked: boolean) => updateSetting("record_aram", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_arena" className="cursor-pointer">
                Arena
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                2v2v2v2 battle arena mode
              </p>
            </div>
            <Switch
              id="record_arena"
              checked={settings.record_arena}
              onCheckedChange={(checked: boolean) => updateSetting("record_arena", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_special" className="cursor-pointer">
                Special Events
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                URF, One for All, and other rotating modes
              </p>
            </div>
            <Switch
              id="record_special"
              checked={settings.record_special}
              onCheckedChange={(checked: boolean) => updateSetting("record_special", checked)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Practice Modes */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Practice Modes</CardTitle>
          <CardDescription>
            Custom games and practice tool
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_custom" className="cursor-pointer">
                Custom Games
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                Player-created custom matches
              </p>
            </div>
            <Switch
              id="record_custom"
              checked={settings.record_custom}
              onCheckedChange={(checked: boolean) => updateSetting("record_custom", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="flex-1">
              <Label htmlFor="record_practice" className="cursor-pointer">
                Practice Tool
              </Label>
              <p className="text-xs text-muted-foreground mt-1">
                Solo practice environment
              </p>
            </div>
            <Switch
              id="record_practice"
              checked={settings.record_practice}
              onCheckedChange={(checked: boolean) => updateSetting("record_practice", checked)}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
