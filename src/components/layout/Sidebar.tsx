import { Home, Film, Video, Settings, LogOut, Sparkles, Youtube, Grid3x3 } from 'lucide-react';
import { Link } from '@tanstack/react-router';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useAuthStore } from '@/lib/auth';
import { useTranslation } from 'react-i18next';

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className = '' }: SidebarProps) {
  const { user, isAuthenticated, logout } = useAuthStore();
  const { t } = useTranslation();

  const navItems = [
    { path: '/', label: t('nav.dashboard'), icon: Home },
    { path: '/games', label: t('nav.games'), icon: Film },
    { path: '/editor', label: t('nav.editor'), icon: Video },
    { path: '/auto-edit', label: t('nav.autoEdit'), icon: Sparkles, badge: t('nav.pro') },
    { path: '/results', label: t('nav.results'), icon: Grid3x3 },
    { path: '/youtube', label: t('nav.youtube'), icon: Youtube, badge: t('nav.pro') },
    { path: '/settings', label: t('nav.settings'), icon: Settings },
  ];

  return (
    <aside className={`w-64 bg-card border-r flex flex-col ${className}`}>
      {/* Logo and Brand */}
      <div className="p-6 border-b">
        <h1 className="text-2xl font-bold">LoLShorts</h1>
        <p className="text-sm text-muted-foreground">{t('app.tagline')}</p>
      </div>

      {/* User Info */}
      {isAuthenticated && user && (
        <div className="p-4 border-b bg-muted/50">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium truncate">{user.email}</span>
            <Badge variant={user.tier === 'Pro' ? 'default' : 'secondary'} className="text-xs">
              {user.tier || 'FREE'}
            </Badge>
          </div>
          {user.tier === 'Free' && (
            <Button variant="outline" size="sm" className="w-full text-xs">
              {t('auth.upgradeToPro')} ⭐
            </Button>
          )}
        </div>
      )}

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-1">
        {navItems.map((item) => (
          <Link
            key={item.path}
            to={item.path}
            className="flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors hover:bg-accent"
            activeProps={{
              className: 'bg-accent text-accent-foreground'
            }}
          >
            <item.icon className="w-5 h-5" />
            <span className="flex-1">{item.label}</span>
            {item.badge && (
              <Badge variant="secondary" className="text-xs">
                {item.badge}
              </Badge>
            )}
          </Link>
        ))}
      </nav>

      {/* Footer Actions */}
      <div className="p-4 border-t space-y-2">
        {isAuthenticated ? (
          <Button
            variant="outline"
            size="sm"
            className="w-full"
            onClick={() => logout()}
          >
            <LogOut className="w-4 h-4 mr-2" />
            {t('auth.logout')}
          </Button>
        ) : (
          <Button variant="outline" size="sm" className="w-full">
            {t('auth.loginSignup')}
          </Button>
        )}
      </div>
    </aside>
  );
}
