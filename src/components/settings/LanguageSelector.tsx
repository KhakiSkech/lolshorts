import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { languages } from '@/i18n';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Globe, Loader2 } from 'lucide-react';

// LoL Region to flag mapping
const regionFlags: Record<string, string> = {
  'NA': '🇺🇸',
  'EUW': '🇪🇺',
  'EUNE': '🇪🇺',
  'OCE': '🇦🇺',
  'KR': '🇰🇷',
  'JP': '🇯🇵',
  'CN': '🇨🇳',
  'TW': '🇹🇼',
  'HK': '🇭🇰',
  'MO': '🇲🇴',
  'BR': '🇧🇷',
  'LAN': '🇲🇽',
  'LAS': '🇦🇷',
  'TR': '🇹🇷',
  'RU': '🇷🇺',
  'VN': '🇻🇳',
  'TH': '🇹🇭',
  'PH': '🇵🇭',
};

export function LanguageSelector() {
  const { i18n, t } = useTranslation();
  const [isChanging, setIsChanging] = useState(false);

  const handleLanguageChange = async (languageCode: string) => {
    // Don't allow changing while already loading
    if (isChanging) return;

    try {
      setIsChanging(true);

      // Change language - this will trigger lazy loading via LazyBackend
      await i18n.changeLanguage(languageCode);

      // Persist selection
      localStorage.setItem('selectedLanguage', languageCode);
    } catch (error) {
      console.error('Failed to change language:', error);
      // Could show a toast notification here
    } finally {
      setIsChanging(false);
    }
  };

  const currentLanguage = languages.find(lang => lang.code === i18n.language) || languages[0];

  // Get unique region flags for current language
  const getRegionFlags = (regions: string[]) => {
    return regions.map(region => regionFlags[region] || '🌍').join(' ');
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Globe className="w-6 h-6" />
          {t('settings.general.language')}
        </CardTitle>
        <CardDescription>
          Choose your preferred language / 언어 선택
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="language-select">
            {t('settings.general.selectLanguage')}
          </Label>
          <Select
            value={i18n.language}
            onValueChange={handleLanguageChange}
            disabled={isChanging}
          >
            <SelectTrigger id="language-select" className="w-full">
              <SelectValue>
                <div className="flex items-center gap-2">
                  {isChanging ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span className="text-muted-foreground">Loading...</span>
                    </>
                  ) : (
                    <>
                      <span className="text-lg">{currentLanguage.flag}</span>
                      <span>{currentLanguage.nativeName}</span>
                      <span className="text-muted-foreground text-sm">
                        ({currentLanguage.name})
                      </span>
                    </>
                  )}
                </div>
              </SelectValue>
            </SelectTrigger>
            <SelectContent className="max-h-[400px]">
              {languages.map((language) => (
                <SelectItem
                  key={language.code}
                  value={language.code}
                  className="cursor-pointer"
                >
                  <div className="flex items-center gap-2 py-1">
                    <span className="text-lg">{language.flag}</span>
                    <div className="flex flex-col">
                      <span className="font-medium">{language.nativeName}</span>
                      <span className="text-xs text-muted-foreground flex items-center gap-1">
                        <span>{language.name}</span>
                        <span>•</span>
                        <span>{getRegionFlags(language.regions)}</span>
                      </span>
                    </div>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="text-sm text-muted-foreground bg-muted p-3 rounded-md">
          <p className="font-medium mb-1">🌍 {t('settings.general.language')} - Auto-Detection</p>
          <p>
            The app automatically detects your system language on first launch.
            You can change it anytime here.
          </p>
          <p className="mt-1">
            앱이 처음 실행 시 자동으로 시스템 언어를 감지합니다.
            언제든지 여기서 변경할 수 있습니다.
          </p>
        </div>

        <div className="text-xs text-muted-foreground flex items-center gap-2">
          <span>{currentLanguage.flag} <strong>{currentLanguage.nativeName}</strong></span>
          <span>•</span>
          <span>LoL: {getRegionFlags(currentLanguage.regions)}</span>
        </div>
      </CardContent>
    </Card>
  );
}
