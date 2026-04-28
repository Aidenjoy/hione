import { Routes, Route, Navigate } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Component, ErrorInfo, ReactNode, useEffect } from 'react'
import Layout from './components/Layout'
import LauncherPage from './pages/LauncherPage'
import TaskPanelPage from './pages/TaskPanelPage'
import ToolManagerPage from './pages/ToolManagerPage'
import AgentConfigPage from './pages/AgentConfigPage'
import McpPage from './pages/McpPage'
import SkillsPage from './pages/SkillsPage'
import CustomToolsPage from './pages/CustomToolsPage'
import HelpPage from './pages/HelpPage'
import AboutPage from './pages/AboutPage'
import SettingsPage from './pages/SettingsPage'
import './i18n'
import { MENU_CONFIG } from './config/menu'
import { useAppStore } from './lib/store'

function ThemeProvider({ children }: { children: ReactNode }) {
  const theme = useAppStore((state) => state.theme);

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove('light', 'dark');

    if (theme === 'system') {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light';
      root.classList.add(systemTheme);
    } else {
      root.classList.add(theme);
    }
  }, [theme]);

  return <>{children}</>;
}

class ErrorBoundary extends Component<{ children: ReactNode }, { hasError: boolean; error: Error | null }> {
  constructor(props: { children: ReactNode }) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Uncaught error:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-8 bg-red-50 text-red-900 min-h-screen font-mono">
          <h1 className="text-2xl font-bold mb-4">React Crash Detected</h1>
          <pre className="bg-red-100 p-4 rounded">{this.state.error?.toString()}</pre>
          <button 
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-red-600 text-white rounded"
          >
            Reload App
          </button>
        </div>
      )
    }
    return this.props.children
  }
}

const queryClient = new QueryClient()

export default function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider>
        <QueryClientProvider client={queryClient}>
          <Routes>
            <Route element={<Layout />}>
              <Route path="/" element={<Navigate to="/launcher" replace />} />
              {MENU_CONFIG.launcher && <Route path="/launcher" element={<LauncherPage />} />}
              {MENU_CONFIG.tasks && <Route path="/tasks" element={<TaskPanelPage />} />}
              {MENU_CONFIG.tools && <Route path="/tools" element={<ToolManagerPage />} />}
              {MENU_CONFIG.agents && <Route path="/agents" element={<AgentConfigPage />} />}
              {MENU_CONFIG.mcp && <Route path="/mcp" element={<McpPage />} />}
              {MENU_CONFIG.skills && <Route path="/skills" element={<SkillsPage />} />}
              {MENU_CONFIG.customTools && <Route path="/custom-tools" element={<CustomToolsPage />} />}
              {MENU_CONFIG.help && <Route path="/help" element={<HelpPage />} />}
              {MENU_CONFIG.about && <Route path="/about" element={<AboutPage />} />}
              {MENU_CONFIG.settings && <Route path="/settings" element={<SettingsPage />} />}
              <Route path="*" element={<Navigate to="/launcher" replace />} />
            </Route>
          </Routes>
        </QueryClientProvider>
      </ThemeProvider>
    </ErrorBoundary>
  )
}
