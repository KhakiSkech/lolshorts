import { useState } from 'react';
import { Home, Film, Video, Settings, LogOut, Sparkles, Youtube, Grid3x3, RotateCcw } from 'lucide-react';
import { Link } from '@tanstack/react-router';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useAuthStore } from '@/lib/auth';
import { useTranslation } from 'react-i18next';
import { AuthModal } from '@/components/auth/AuthModal';

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className = '' }: SidebarProps) {
  const { user, isAuthenticated, logout } = useAuthStore();
  const { t } = useTranslation();
  const [authModalOpen, setAuthModalOpen] = useState(false);

  const navItems = [
    { path: '/', label: t('nav.dashboard'), icon: Home },
    { path: '/games', label: t('nav.games'), icon: Film },
    { path: '/replays', label: "Replays", icon: RotateCcw },
    { path: '/editor', label: t('nav.editor'), icon: Video },
    { path: '/auto-edit', label: t('nav.autoEdit'), icon: Sparkles, badge: t('nav.pro'), proRequired: true },
    { path: '/results', label: t('nav.results'), icon: Grid3x3 },
    { path: '/youtube', label: t('nav.youtube'), icon: Youtube, badge: t('nav.pro'), proRequired: true },
    { path: '/settings', label: t('nav.settings'), icon: Settings },
  ];

  return (
    <aside className={`w-64 bg-card border-r flex flex-col ${className} hidden md:flex`}>
      {/* Logo and Brand */}
      <div className="p-6 border-b bg-gradient-to-r from-primary/10 to-primary/5">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-primary rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-lg">LS</span>
          </div>
          <div>
            <h1 className="text-xl font-bold">LoLShorts</h1>
            <p className="text-xs text-muted-foreground">{t('app.tagline')}</p>
          </div>
        </div>
      </div>

      {/* User Info */}
      {isAuthenticated && user && (
        <div className="p-4 border-b bg-muted/30">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 bg-secondary rounded-full flex items-center justify-center">
                  <span className="text-xs font-medium">
                    {user.email?.charAt(0)?.toUpperCase()}
                  </span>
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{user.email}</p>
                </div>
              </div>
              <Badge
                variant={user.tier === 'PRO' ? 'default' : 'secondary'}
                className={user.tier === 'PRO' ? 'bg-accent-pro text-accent-pro-foreground border-accent-pro' : ''}
              >
                {user.tier || 'FREE'}
              </Badge>
            </div>
            {user.tier === 'FREE' && (
              <Button
                variant="outline"
                size="sm"
                className="w-full text-xs border-accent-pro text-accent-pro hover:bg-accent-pro/10 hover:text-accent-pro-hover"
                onClick={() => setAuthModalOpen(true)}
              >
                {t('auth.upgradeToPro')}
              </Button>
            )}
          </div>
        </div>
      )}

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-1 overflow-y-auto">
        {navItems.map((item) => {
          const isProRequired = item.proRequired && user?.tier !== 'PRO';
          return (
            <Link
              key={item.path}
              to={item.path}
              className={`
                flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium
                transition-all duration-200 group
                ${isProRequired
                  ? 'opacity-60 cursor-not-allowed'
                  : 'hover:bg-accent hover:text-accent-foreground'
                }
              `}
              activeProps={{
                className: 'bg-accent text-accent-foreground shadow-sm'
              }}
              onClick={isProRequired ? (e) => {
                e.preventDefault();
                setAuthModalOpen(true);
              } : undefined}
            >
              <div className={`relative ${isProRequired ? 'grayscale' : ''}`}>
                <item.icon className="w-5 h-5" />
                {isProRequired && (
                  <div className="absolute -top-1 -right-1 w-3 h-3 bg-yellow-500 rounded-full flex items-center justify-center">
                    <span className="text-[8px]">⭐</span>
                  </div>
                )}
              </div>
              <span className="flex-1">{item.label}</span>
              {item.badge && (
                <Badge
                  variant={item.proRequired && user?.tier !== 'PRO' ? "outline" : "secondary"}
                  className={`text-xs ${item.proRequired && user?.tier !== 'PRO' ? 'border-yellow-600 text-yellow-600' : ''}`}
                >
                  {item.badge}
                </Badge>
              )}
            </Link>
          );
        })}
      </nav>

      {/* Footer Actions */}
      <div className="p-4 border-t bg-muted/20">
        {isAuthenticated ? (
          <div className="space-y-2">
            <div className="text-xs text-muted-foreground text-center">
              {user?.tier === 'PRO' ? '✨ Pro Member' : 'Free Account'}
            </div>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() => logout()}
              data-testid="logout-button"
            >
              <LogOut className="w-4 h-4 mr-2" />
              {t('auth.logout')}
            </Button>
          </div>
        ) : (
          <Button
            className="w-full"
            onClick={() => setAuthModalOpen(true)}
            data-testid="sidebar-login-button"
          >
            {t('auth.loginSignup')}
          </Button>
        )}
      </div>

      {/* Auth Modal */}
      <AuthModal
        open={authModalOpen}
        onClose={() => setAuthModalOpen(false)}
        defaultMode="login"
      />
    </aside>
  );
}
