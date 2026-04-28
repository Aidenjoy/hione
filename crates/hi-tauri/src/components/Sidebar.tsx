import { NavLink } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { 
  Rocket, ClipboardList, Wrench, Bot, Plug, 
  Package, Settings, HelpCircle, Cpu, Info
} from 'lucide-react'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { MENU_CONFIG, type MenuKey } from '@/config/menu'

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function Sidebar() {
  const { t } = useTranslation()

  const navItems = [
    { to: '/launcher', icon: Rocket, label: t('nav.launcher'), key: 'launcher' as MenuKey },
    { to: '/tasks', icon: ClipboardList, label: t('nav.tasks'), key: 'tasks' as MenuKey },
    { to: '/tools', icon: Wrench, label: t('nav.tools'), key: 'tools' as MenuKey },
    { to: '/agents', icon: Bot, label: t('nav.agents'), key: 'agents' as MenuKey },
    { to: '/mcp', icon: Plug, label: t('nav.mcp'), key: 'mcp' as MenuKey },
    { to: '/skills', icon: Package, label: t('nav.skills'), key: 'skills' as MenuKey },
    { to: '/custom-tools', icon: Cpu, label: t('nav.customTools'), key: 'customTools' as MenuKey },
    { to: '/help', icon: HelpCircle, label: t('nav.help'), key: 'help' as MenuKey },
    { to: '/about', icon: Info, label: t('nav.about'), key: 'about' as MenuKey },
    { to: '/settings', icon: Settings, label: t('nav.settings'), key: 'settings' as MenuKey },
  ].filter(item => MENU_CONFIG[item.key]);

  return (
    <aside className="w-64 bg-gray-100 dark:bg-gray-900 h-full flex flex-col border-r border-gray-200 dark:border-gray-800">
      <div className="p-6">
        <div className="text-3xl font-black tracking-tighter italic text-gray-900 dark:text-white">hione</div>
        <div className="text-xs font-black text-blue-500 uppercase tracking-[0.3em] mt-1 pl-0.5">all in one ai tools</div>
      </div>
      <nav className="flex-1 overflow-y-auto px-4 space-y-2">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) => cn(
              "flex items-center space-x-3 p-3 rounded-lg transition-colors",
              isActive 
                ? "bg-blue-500 text-white" 
                : "text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-800"
            )}
          >
            <item.icon size={20} />
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>
    </aside>
  )
}
