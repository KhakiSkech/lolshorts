import { ResultsViewer } from '@/components/results/ResultsViewer';
import { ProtectedFeature } from '@/components/auth/ProtectedFeature';
import { useTranslation } from 'react-i18next';

export function Results() {
  const { t } = useTranslation();

  return (
    <ProtectedFeature requiresPro={false} featureName={t('results.title')}>
      <div className="h-full flex flex-col overflow-hidden">
        <ResultsViewer />
      </div>
    </ProtectedFeature>
  );
}
