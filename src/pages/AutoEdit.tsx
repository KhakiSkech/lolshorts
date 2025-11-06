import { AutoEditPanel } from '@/components/editor/AutoEditPanel';
import { ProtectedFeature } from '@/components/auth/ProtectedFeature';

export function AutoEdit() {
  return (
    <ProtectedFeature requiresPro={true} featureName="Auto-Edit">
      <div className="h-full flex flex-col overflow-hidden">
        <AutoEditPanel />
      </div>
    </ProtectedFeature>
  );
}
