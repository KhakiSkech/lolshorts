import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { LazyBackend } from './i18n/lazyBackend';

/**
 * Translation files are now loaded lazily using dynamic imports.
 * Only the active language is loaded, reducing initial bundle size by ~75 KB.
 *
 * Benefits:
 * - Initial load: Only English (~4 KB) instead of all 20 languages (~80 KB)
 * - Language switching: Loads on-demand with ~200ms latency
 * - Bundle optimization: Vite creates separate chunks for each language
 */

// Language configuration
export const languages = [
  { code: 'en', name: 'English', nativeName: 'English', flag: '🇺🇸', regions: ['NA', 'EUW', 'EUNE', 'OCE'] },
  { code: 'ko', name: 'Korean', nativeName: '한국어', flag: '🇰🇷', regions: ['KR'] },
  { code: 'ja', name: 'Japanese', nativeName: '日本語', flag: '🇯🇵', regions: ['JP'] },
  { code: 'zh-CN', name: 'Simplified Chinese', nativeName: '简体中文', flag: '🇨🇳', regions: ['CN'] },
  { code: 'zh-TW', name: 'Traditional Chinese', nativeName: '繁體中文', flag: '🇹🇼', regions: ['TW', 'HK', 'MO'] },
  { code: 'de', name: 'German', nativeName: 'Deutsch', flag: '🇩🇪', regions: ['EUW'] },
  { code: 'fr', name: 'French', nativeName: 'Français', flag: '🇫🇷', regions: ['EUW'] },
  { code: 'es', name: 'Spanish', nativeName: 'Español', flag: '🇪🇸', regions: ['EUW', 'LAN', 'LAS'] },
  { code: 'it', name: 'Italian', nativeName: 'Italiano', flag: '🇮🇹', regions: ['EUW'] },
  { code: 'pt-BR', name: 'Portuguese (Brazil)', nativeName: 'Português (Brasil)', flag: '🇧🇷', regions: ['BR'] },
  { code: 'pl', name: 'Polish', nativeName: 'Polski', flag: '🇵🇱', regions: ['EUNE'] },
  { code: 'tr', name: 'Turkish', nativeName: 'Türkçe', flag: '🇹🇷', regions: ['TR'] },
  { code: 'ru', name: 'Russian', nativeName: 'Русский', flag: '🇷🇺', regions: ['RU'] },
  { code: 'cs', name: 'Czech', nativeName: 'Čeština', flag: '🇨🇿', regions: ['EUNE'] },
  { code: 'el', name: 'Greek', nativeName: 'Ελληνικά', flag: '🇬🇷', regions: ['EUNE'] },
  { code: 'hu', name: 'Hungarian', nativeName: 'Magyar', flag: '🇭🇺', regions: ['EUNE'] },
  { code: 'ro', name: 'Romanian', nativeName: 'Română', flag: '🇷🇴', regions: ['EUNE'] },
  { code: 'vi', name: 'Vietnamese', nativeName: 'Tiếng Việt', flag: '🇻🇳', regions: ['VN'] },
  { code: 'th', name: 'Thai', nativeName: 'ไทย', flag: '🇹🇭', regions: ['TH'] },
  { code: 'fil', name: 'Filipino', nativeName: 'Filipino', flag: '🇵🇭', regions: ['PH'] },
];

i18n
  .use(LazyBackend) // Use lazy loading backend
  .use(LanguageDetector) // Detect user language
  .use(initReactI18next) // Pass i18n to React
  .init({
    // No resources - loaded dynamically by LazyBackend
    fallbackLng: 'en', // Default language
    debug: false,

    // Supported languages (used by LanguageDetector)
    supportedLngs: [
      'en', 'ko', 'ja', 'zh-CN', 'zh-TW',
      'de', 'fr', 'es', 'it', 'pt-BR',
      'pl', 'tr', 'ru', 'cs', 'el',
      'hu', 'ro', 'vi', 'th', 'fil',
    ],

    // Only load 'translation' namespace
    ns: ['translation'],
    defaultNS: 'translation',

    interpolation: {
      escapeValue: false, // React already escapes
    },

    detection: {
      // Order of language detection
      order: ['localStorage', 'navigator', 'htmlTag'],
      caches: ['localStorage'],
    },

    react: {
      useSuspense: false, // Don't use Suspense for language loading
    },

    backend: {
      loadPath: '/locales/{{lng}}/{{ns}}.json', // Path pattern (not used, just for typing)
    },
  });

export default i18n;
