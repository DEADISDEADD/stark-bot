import { NavLink, Outlet } from 'react-router-dom';
import {
  BookOpen,
  Rocket,
  Box,
  Wrench,
  Zap,
  Monitor,
  Clock,
  Settings,
  Code,
  Brain
} from 'lucide-react';

const docsNav = [
  { path: '/docs', label: 'Overview', icon: BookOpen, exact: true },
  { path: '/docs/getting-started', label: 'Getting Started', icon: Rocket },
  { path: '/docs/architecture', label: 'Architecture', icon: Box },
  { path: '/docs/api', label: 'API Reference', icon: Code },
  { path: '/docs/tools', label: 'Tools', icon: Wrench },
  { path: '/docs/skills', label: 'Skills', icon: Zap },
  { path: '/docs/channels', label: 'Channels', icon: Monitor },
  { path: '/docs/scheduling', label: 'Scheduling', icon: Clock },
  { path: '/docs/memories', label: 'Memories', icon: Brain },
  { path: '/docs/configuration', label: 'Configuration', icon: Settings },
];

export default function DocsLayout() {
  return (
    <div className="flex min-h-screen">
      {/* Docs Sidebar */}
      <aside className="w-64 bg-slate-800/50 border-r border-slate-700 p-4">
        <div className="mb-6">
          <h2 className="text-lg font-semibold text-slate-200">Documentation</h2>
          <p className="text-sm text-slate-400">StarkBot Reference</p>
        </div>
        <nav className="space-y-1">
          {docsNav.map(({ path, label, icon: Icon, exact }) => (
            <NavLink
              key={path}
              to={path}
              end={exact}
              className={({ isActive }) =>
                `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                  isActive
                    ? 'bg-stark-500/20 text-stark-400'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-slate-700/50'
                }`
              }
            >
              <Icon className="w-4 h-4" />
              {label}
            </NavLink>
          ))}
        </nav>
      </aside>

      {/* Docs Content */}
      <main className="flex-1 p-8 overflow-auto">
        <div className="max-w-4xl mx-auto">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
