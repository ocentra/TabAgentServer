import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';

interface BreadcrumbItem {
  label: string;
  href?: string;
  icon?: React.ReactNode;
}

interface BreadcrumbProps {
  items?: BreadcrumbItem[];
  className?: string;
}

const routeLabels: Record<string, string> = {
  '/': 'Dashboard',
  '/logs': 'Logs',
  '/models': 'Models',
  '/database': 'Database',
  '/settings': 'Settings',
  '/api': 'API Explorer',
  '/webrtc': 'WebRTC Console',
  '/chat': 'AI Chat',
  '/knowledge': 'Knowledge Base',
  '/knowledge-graph': 'Knowledge Graph',
};

const Breadcrumb: React.FC<BreadcrumbProps> = ({ items, className }) => {
  const location = useLocation();

  // Auto-generate breadcrumbs from current route if items not provided
  const breadcrumbItems = items || generateBreadcrumbs(location.pathname);

  if (breadcrumbItems.length === 0) return null;

  return (
    <nav className={cn('flex items-center space-x-1 text-sm text-muted-foreground', className)}>
      {breadcrumbItems.map((item, index) => (
        <React.Fragment key={index}>
          {index > 0 && (
            <svg
              className="w-4 h-4 text-muted-foreground/50"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          )}
          <div className="flex items-center space-x-1">
            {item.icon && <span className="w-4 h-4">{item.icon}</span>}
            {item.href && index < breadcrumbItems.length - 1 ? (
              <Link
                to={item.href}
                className="hover:text-foreground transition-colors"
              >
                {item.label}
              </Link>
            ) : (
              <span className={index === breadcrumbItems.length - 1 ? 'text-foreground font-medium' : ''}>
                {item.label}
              </span>
            )}
          </div>
        </React.Fragment>
      ))}
    </nav>
  );
};

function generateBreadcrumbs(pathname: string): BreadcrumbItem[] {
  const paths = pathname.split('/').filter(Boolean);
  const breadcrumbs: BreadcrumbItem[] = [];

  // Always include home
  breadcrumbs.push({
    label: 'Dashboard',
    href: '/',
  });

  // Add intermediate paths
  let currentPath = '';
  paths.forEach((path, index) => {
    currentPath += `/${path}`;
    const label = routeLabels[currentPath] || path.charAt(0).toUpperCase() + path.slice(1);
    
    breadcrumbs.push({
      label,
      href: index === paths.length - 1 ? undefined : currentPath, // Last item has no href
    });
  });

  return breadcrumbs;
}

export { Breadcrumb };
export type { BreadcrumbItem, BreadcrumbProps };