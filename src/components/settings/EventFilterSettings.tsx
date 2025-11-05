import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface EventFilterSettings {
  record_kills: boolean;
  record_multikills: boolean;
  record_first_blood: boolean;
  record_deaths: boolean;
  record_shutdown: boolean;
  record_assists: boolean;
  record_dragon: boolean;
  record_baron: boolean;
  record_elder: boolean;
  record_herald: boolean;
  record_turret: boolean;
  record_inhibitor: boolean;
  record_nexus: boolean;
  record_ace: boolean;
  record_game_end: boolean;
  record_steal: boolean;
  min_priority: number;
}

interface EventFilterSettingsProps {
  settings: EventFilterSettings;
  onChange: (settings: EventFilterSettings) => void;
}

export function EventFilterSettings({ settings, onChange }: EventFilterSettingsProps) {
  const updateSetting = (key: keyof EventFilterSettings, value: boolean | number) => {
    onChange({ ...settings, [key]: value });
  };

  const applyPreset = (preset: "highlights" | "everything" | "minimal") => {
    switch (preset) {
      case "highlights":
        onChange({
          record_kills: true,
          record_multikills: true,
          record_first_blood: true,
          record_deaths: false,
          record_shutdown: false,
          record_assists: false,
          record_dragon: true,
          record_baron: true,
          record_elder: true,
          record_herald: true,
          record_turret: false,
          record_inhibitor: true,
          record_nexus: true,
          record_ace: true,
          record_game_end: true,
          record_steal: true,
          min_priority: 2,
        });
        break;
      case "everything":
        onChange({
          record_kills: true,
          record_multikills: true,
          record_first_blood: true,
          record_deaths: true,
          record_shutdown: true,
          record_assists: true,
          record_dragon: true,
          record_baron: true,
          record_elder: true,
          record_herald: true,
          record_turret: true,
          record_inhibitor: true,
          record_nexus: true,
          record_ace: true,
          record_game_end: true,
          record_steal: true,
          min_priority: 1,
        });
        break;
      case "minimal":
        onChange({
          record_kills: false,
          record_multikills: true,
          record_first_blood: true,
          record_deaths: false,
          record_shutdown: false,
          record_assists: false,
          record_dragon: false,
          record_baron: true,
          record_elder: true,
          record_herald: false,
          record_turret: false,
          record_inhibitor: true,
          record_nexus: true,
          record_ace: true,
          record_game_end: true,
          record_steal: true,
          min_priority: 3,
        });
        break;
    }
  };

  const getPriorityLabel = (priority: number): string => {
    const labels = {
      1: "All Events",
      2: "Important Events",
      3: "High Priority",
      4: "Critical Moments",
      5: "Epic Plays Only",
    };
    return labels[priority as keyof typeof labels] || "Custom";
  };

  return (
    <div className="space-y-6">
      {/* Presets */}
      <div>
        <h3 className="text-sm font-semibold mb-3">Quick Presets</h3>
        <div className="flex gap-2 flex-wrap">
          <Button variant="outline" size="sm" onClick={() => applyPreset("highlights")}>
            Highlights Only
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("everything")}>
            Everything
          </Button>
          <Button variant="outline" size="sm" onClick={() => applyPreset("minimal")}>
            Minimal (Epic Only)
          </Button>
        </div>
      </div>

      {/* Priority Filter */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Priority Filter</CardTitle>
          <CardDescription>
            Filter events by importance level (1 = all, 5 = epic only)
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label>Minimum Priority</Label>
              <Badge variant="secondary">{getPriorityLabel(settings.min_priority)}</Badge>
            </div>
            <Slider
              value={[settings.min_priority]}
              onValueChange={([value]) => updateSetting("min_priority", value)}
              min={1}
              max={5}
              step={1}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>All</span>
              <span>Important</span>
              <span>High</span>
              <span>Critical</span>
              <span>Epic</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Kill Events */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Kill Events</CardTitle>
          <CardDescription>
            Record kills, deaths, and assists
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="record_kills" className="flex-1 cursor-pointer">
              Kills
            </Label>
            <Switch
              id="record_kills"
              checked={settings.record_kills}
              onCheckedChange={(checked: boolean) => updateSetting("record_kills", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_multikills" className="flex-1 cursor-pointer">
              Multikills (Double, Triple, Quadra, Penta)
            </Label>
            <Switch
              id="record_multikills"
              checked={settings.record_multikills}
              onCheckedChange={(checked: boolean) => updateSetting("record_multikills", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_first_blood" className="flex-1 cursor-pointer">
              First Blood
            </Label>
            <Switch
              id="record_first_blood"
              checked={settings.record_first_blood}
              onCheckedChange={(checked: boolean) => updateSetting("record_first_blood", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_deaths" className="flex-1 cursor-pointer">
              Deaths
            </Label>
            <Switch
              id="record_deaths"
              checked={settings.record_deaths}
              onCheckedChange={(checked: boolean) => updateSetting("record_deaths", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_shutdown" className="flex-1 cursor-pointer">
              Shutdown (Ending enemy killing spree)
            </Label>
            <Switch
              id="record_shutdown"
              checked={settings.record_shutdown}
              onCheckedChange={(checked: boolean) => updateSetting("record_shutdown", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_assists" className="flex-1 cursor-pointer">
              Assists
            </Label>
            <Switch
              id="record_assists"
              checked={settings.record_assists}
              onCheckedChange={(checked: boolean) => updateSetting("record_assists", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_ace" className="flex-1 cursor-pointer">
              Ace (Team kills all enemies)
            </Label>
            <Switch
              id="record_ace"
              checked={settings.record_ace}
              onCheckedChange={(checked: boolean) => updateSetting("record_ace", checked)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Objective Events */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Objective Events</CardTitle>
          <CardDescription>
            Record dragon, baron, herald, and steal attempts
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="record_dragon" className="flex-1 cursor-pointer">
              Dragon
            </Label>
            <Switch
              id="record_dragon"
              checked={settings.record_dragon}
              onCheckedChange={(checked: boolean) => updateSetting("record_dragon", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_baron" className="flex-1 cursor-pointer">
              Baron Nashor
            </Label>
            <Switch
              id="record_baron"
              checked={settings.record_baron}
              onCheckedChange={(checked: boolean) => updateSetting("record_baron", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_elder" className="flex-1 cursor-pointer">
              Elder Dragon
            </Label>
            <Switch
              id="record_elder"
              checked={settings.record_elder}
              onCheckedChange={(checked: boolean) => updateSetting("record_elder", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_herald" className="flex-1 cursor-pointer">
              Rift Herald
            </Label>
            <Switch
              id="record_herald"
              checked={settings.record_herald}
              onCheckedChange={(checked: boolean) => updateSetting("record_herald", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_steal" className="flex-1 cursor-pointer">
              Objective Steals
            </Label>
            <Switch
              id="record_steal"
              checked={settings.record_steal}
              onCheckedChange={(checked: boolean) => updateSetting("record_steal", checked)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Structure Events */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Structure Events</CardTitle>
          <CardDescription>
            Record tower, inhibitor, and nexus destruction
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="record_turret" className="flex-1 cursor-pointer">
              Turrets
            </Label>
            <Switch
              id="record_turret"
              checked={settings.record_turret}
              onCheckedChange={(checked: boolean) => updateSetting("record_turret", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_inhibitor" className="flex-1 cursor-pointer">
              Inhibitors
            </Label>
            <Switch
              id="record_inhibitor"
              checked={settings.record_inhibitor}
              onCheckedChange={(checked: boolean) => updateSetting("record_inhibitor", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_nexus" className="flex-1 cursor-pointer">
              Nexus
            </Label>
            <Switch
              id="record_nexus"
              checked={settings.record_nexus}
              onCheckedChange={(checked: boolean) => updateSetting("record_nexus", checked)}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="record_game_end" className="flex-1 cursor-pointer">
              Game End
            </Label>
            <Switch
              id="record_game_end"
              checked={settings.record_game_end}
              onCheckedChange={(checked: boolean) => updateSetting("record_game_end", checked)}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
