import { ReactNode } from 'react';

interface EditorLayoutProps {
  clipLibrary: ReactNode;
  videoPreview: ReactNode;
  compositionSettings: ReactNode;
  timeline: ReactNode;
}

export function EditorLayout({
  clipLibrary,
  videoPreview,
  compositionSettings,
  timeline,
}: EditorLayoutProps) {
  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Top Section: 3 columns */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left: Clip Library */}
        <aside className="w-80 border-r bg-card overflow-y-auto">
          {clipLibrary}
        </aside>

        {/* Center: Video Preview */}
        <main className="flex-1 flex flex-col overflow-hidden">
          {videoPreview}
        </main>

        {/* Right: Composition Settings */}
        <aside className="w-80 border-l bg-card overflow-y-auto">
          {compositionSettings}
        </aside>
      </div>

      {/* Bottom: Timeline */}
      <div className="h-64 border-t bg-card">
        {timeline}
      </div>
    </div>
  );
}
