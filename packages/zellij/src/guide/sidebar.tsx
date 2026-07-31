'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import type { SidebarSection } from '../content/types';

export function GuideSidebar({ sidebar }: { sidebar: SidebarSection[] }) {
  const pathname = usePathname() || '';

  return (
    <nav className="zellij-guide-sidebar-nav" aria-label="Guide">
      {sidebar.map((section) => (
        <div className="zellij-guide-sidebar-section" key={section.label}>
          <p className="zellij-guide-sidebar-heading">{section.label}</p>
          <ul>
            {section.items.map((item) => {
              const active = pathname === item.route;
              return (
                <li key={item.route}>
                  <Link
                    className="zellij-guide-sidebar-link"
                    href={item.route}
                    data-active={active}
                    {...(active ? { 'aria-current': 'page' as const } : {})}
                  >
                    {item.label}
                  </Link>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </nav>
  );
}
