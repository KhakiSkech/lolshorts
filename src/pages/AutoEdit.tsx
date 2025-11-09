import { AutoEditPanel } from '@/components/editor/AutoEditPanel';
import { ProtectedFeature } from '@/components/auth/ProtectedFeature';
import { useTranslation } from 'react-i18next';

export function AutoEdit() {
  const { t } = useTranslation();

  return (
    <ProtectedFeature requiresPro={false} featureName={t('autoEdit.title')}>
      <div className="h-full flex flex-col overflow-hidden">
        <AutoEditPanel />
      </div>
    </ProtectedFeature>
  );
}
