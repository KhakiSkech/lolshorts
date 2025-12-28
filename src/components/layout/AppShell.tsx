import { ReactNode, useState } from 'react';
import { Sidebar } from './Sidebar';
import { Button } from '@/components/ui/button';
import { Menu, X } from 'lucide-react';

interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <div className="flex h-screen w-screen overflow-hidden">
      {/* Desktop Sidebar */}
      <Sidebar className="hidden md:flex" />

      {/* Mobile Menu Button */}
      <div className="md:hidden fixed top-4 left-4 z-50">
        <Button
          variant="outline"
          size="sm"
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
          className="bg-background/80 backdrop-blur-sm border-2"
        >
          {mobileMenuOpen ? (
            <X className="h-4 w-4" />
          ) : (
            <Menu className="h-4 w-4" />
          )}
        </Button>
      </div>

      {/* Mobile Sidebar Overlay */}
      {mobileMenuOpen && (
        <>
          <div
            className="md:hidden fixed inset-0 bg-black/50 z-40"
            onClick={() => setMobileMenuOpen(false)}
            onKeyDown={(e) => {
              if (e.key === 'Escape') setMobileMenuOpen(false);
            }}
            role="button"
            tabIndex={0}
            aria-label="Close mobile menu"
          />
          <div className="md:hidden fixed inset-y-0 left-0 z-50 w-64 bg-card border-r flex flex-col">
            <Sidebar className="flex" />
          </div>
        </>
      )}

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto bg-background">
        <div className="container mx-auto p-4 md:p-6 max-w-7xl">
          {/* Mobile Header */}
          <div className="md:hidden mb-6 pt-8">
            <h1 className="text-2xl font-bold bg-gradient-to-r from-primary to-primary/80 bg-clip-text text-transparent">
              LoLShorts
            </h1>
            <p className="text-sm text-muted-foreground">
              Automatic League of Legends Highlights
            </p>
          </div>
          {children}
        </div>
      </main>
    </div>
  );
}
