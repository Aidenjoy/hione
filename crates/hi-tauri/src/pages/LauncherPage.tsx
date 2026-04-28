import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { 
  listRecentSessions, 
  launchSession, 
  connectSession, 
  detectSession, 
  killSession,
  listTools 
} from '@/lib/tauri'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { useAppStore } from '@/lib/store'
import { cn } from '@/lib/utils'
import { AgentIcon } from '@/components/AgentIcon'
import { 
  FolderOpen, 
  Play, 
  RefreshCw, 
  Terminal, 
  History, 
  CheckCircle2, 
  AlertCircle,
  Loader2,
  Plug,
  XCircle
} from 'lucide-react'

const TOOL_META: Record<string, { icon: string; label: string }> = {
  claude:   { icon: '🤖', label: 'Claude Code' },
  gemini:   { icon: '💎', label: 'Gemini CLI' },
  opencode: { icon: '🔷', label: 'OpenCode' },
  codex:    { icon: '📦', label: 'Codex CLI' },
  qwen:     { icon: '🌟', label: 'Qwen Code' },
}

function formatRelativeTime(timestamp: number) {
  const diff = Date.now() - timestamp;
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  if (hours < 24) return `${hours} 小时前`;
  if (days === 1) return '昨天';
  return `${days} 天前`;
}

export default function LauncherPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { setWorkDir, setSessionConnected, setSessionInfo } = useAppStore()

  // Selection states
  const [selectedTools, setSelectedTools] = useState<string[]>([])
  const [workDir, setWorkDirInput] = useState('')
  const [autoMode, setAutoMode] = useState(true)
  const [resumeMode, setResumeMode] = useState(true)
  const [launchStatus, setLaunchStatus] = useState<{ type: 'success' | 'error' | null; message: string }>({ type: null, message: '' })

  // Data fetching
  const { data: allTools } = useQuery({ queryKey: ['tools'], queryFn: listTools })
  const { data: recentSessions } = useQuery({ queryKey: ['recentSessions'], queryFn: listRecentSessions })

  // Filter tools to only show those defined in TOOL_META (excludes tmux and others)
  const displayTools = (allTools && allTools.filter(t => t.installed && TOOL_META[t.name]).length > 0)
    ? allTools.filter(t => t.installed && TOOL_META[t.name])
    : Object.keys(TOOL_META).map(name => ({ name, installed: true, version: undefined }))

  // Mutations
  const launchMutation = useMutation({
    mutationFn: () => launchSession(workDir, selectedTools, autoMode, resumeMode),
    onSuccess: () => {
      setLaunchStatus({ type: 'success', message: '会话正在启动...' })
    },
    onError: (err: any) => {
      setLaunchStatus({ type: 'error', message: `启动失败: ${err}` })
    }
  })

  const connectMutation = useMutation({
    mutationFn: async (dir: string) => {
      const active = await detectSession(dir)
      if (active) {
        return connectSession(dir)
      }
      throw new Error('会话已结束，请重新启动')
    },
    onSuccess: (sessionInfo, dir) => {
      setWorkDir(dir)
      setSessionConnected(true)
      setSessionInfo(sessionInfo)
    },
    onError: (err: any) => {
      alert(err.message)
    }
  })

  const killMutation = useMutation({
    mutationFn: (dir: string) => killSession(dir),
    onSuccess: () => {
      setSessionConnected(false)
      queryClient.invalidateQueries({ queryKey: ['recentSessions'] })
    }
  })

  // Listen for session status
  useEffect(() => {
    const unlisten = listen<{ connected: boolean; work_dir: string }>('session://status', async (event) => {
      if (event.payload.connected) {
        setLaunchStatus({ type: 'success', message: '已连接到会话' })
        setWorkDir(event.payload.work_dir)
        setSessionConnected(true)
        queryClient.invalidateQueries({ queryKey: ['recentSessions'] })
        // Fetch session info for status bar
        try {
          const info = await connectSession(event.payload.work_dir)
          setSessionInfo(info)
        } catch {
          // Ignore errors, status bar will handle gracefully
        }
      }
    })
    return () => { unlisten.then(f => f()) }
  }, [queryClient, setWorkDir, setSessionConnected, setSessionInfo])

  const handleBrowse = async () => {
    const selected = await open({ directory: true })
    if (selected && typeof selected === 'string') {
      setWorkDirInput(selected)
    }
  }

  const toggleTool = (name: string) => {
    setSelectedTools(prev => 
      prev.includes(name) ? prev.filter(t => t !== name) : [...prev, name]
    )
  }

  const generatePreview = () => {
    if (selectedTools.length === 0) return 'hi start ...'
    const flags = `${autoMode ? '-a ' : ''}${resumeMode ? '-r ' : ''}`
    return `hi s ${flags.trim()}${flags ? ' ' : ''}${selectedTools.join(',')}`
  }

  return (
    <div className="h-full flex flex-col gap-6 animate-in fade-in duration-500 overflow-hidden">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-black tracking-tight">{t('nav.launcher')}</h1>
        {launchStatus.type && (
          <div className={cn(
            "flex items-center space-x-2 px-3 py-1 rounded-full text-xs font-black uppercase tracking-widest animate-in fade-in slide-in-from-top-1",
            launchStatus.type === 'success' 
              ? "bg-green-50 text-green-600 dark:bg-green-900/20 dark:text-green-400" 
              : "bg-red-50 text-red-600 dark:bg-red-900/20 dark:text-red-400"
          )}>
            {launchStatus.type === 'success' ? <CheckCircle2 size={14} /> : <AlertCircle size={14} />}
            <span>{launchStatus.message}</span>
          </div>
        )}
      </div>

      <div className="flex-1 grid grid-cols-1 lg:grid-cols-5 gap-6 min-h-0 overflow-hidden">
        {/* Left: New Session Config */}
        <section className="lg:col-span-3 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl p-6 shadow-sm flex flex-col gap-6 overflow-y-auto scrollbar-hide">
          <div className="flex items-center space-x-2 border-b border-gray-50 dark:border-gray-800 pb-4">
            <Terminal size={18} className="text-blue-500" />
            <h2 className="text-base font-black uppercase tracking-widest text-gray-400">启动新会话</h2>
          </div>

          <div className="space-y-6 flex-1">
            {/* Work Dir */}
            <div className="space-y-2">
              <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-0.5">工作目录</label>
              <div className="flex space-x-3">
                <input
                  type="text"
                  value={workDir}
                  onChange={(e) => setWorkDirInput(e.target.value)}
                  placeholder="选择项目目录"
                  className="flex-1 bg-gray-50 dark:bg-gray-950 border border-gray-100 dark:border-gray-800 rounded-xl px-4 py-2.5 text-sm font-mono outline-none focus:ring-2 focus:ring-blue-500/20 transition-all"
                />
                <button
                  onClick={handleBrowse}
                  className="flex items-center space-x-2 px-4 py-2.5 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-xl text-xs font-black uppercase tracking-widest transition-colors"
                >
                  <FolderOpen size={16} />
                  <span>浏览</span>
                </button>
              </div>
            </div>

            {/* Tool Selection */}
            <div className="space-y-2">
              <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-0.5">选择工具</label>
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3">
                {displayTools.map(tool => (
                  <button
                    key={tool.name}
                    onClick={() => toggleTool(tool.name)}
                    className={cn(
                      "flex flex-col items-center justify-center py-4 px-2 rounded-2xl border-2 transition-all group relative overflow-hidden bg-white dark:bg-gray-800",
                      selectedTools.includes(tool.name)
                        ? "border-blue-500 bg-blue-50 dark:bg-blue-900/40 text-blue-600 dark:text-blue-400"
                        : "border-gray-50 dark:border-gray-700 hover:border-blue-200 dark:hover:border-blue-800 text-gray-500"
                    )}
                  >
                    <div className="mb-1.5 group-hover:scale-110 transition-transform">
                      <AgentIcon 
                        name={tool.name} 
                        emoji={TOOL_META[tool.name]?.icon || '🛠'} 
                        size={32} 
                      />
                    </div>
                    <span className="text-xs font-black uppercase tracking-tight">{TOOL_META[tool.name]?.label || tool.name}</span>
                    {selectedTools.includes(tool.name) && (
                      <div className="absolute top-1.5 right-1.5">
                        <CheckCircle2 size={12} className="text-blue-500" />
                      </div>
                    )}
                  </button>
                ))}
              </div>
            </div>

            {/* Options & Preview */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-0.5">启动选项</label>
                <div className="flex flex-col gap-2">
                  <label className="flex items-center space-x-2.5 cursor-pointer group bg-gray-50 dark:bg-gray-950 px-3 py-2 rounded-xl border border-transparent hover:border-gray-200 dark:hover:border-gray-800 transition-all">
                    <input
                      type="checkbox"
                      checked={autoMode}
                      onChange={(e) => setAutoMode(e.target.checked)}
                      className="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
                    />
                    <span className="text-xs font-bold text-gray-500 dark:text-gray-400 group-hover:text-black dark:group-hover:text-white uppercase tracking-tight">
                      Auto (-a) 跳过权限确认
                    </span>
                  </label>
                  <label className="flex items-center space-x-2.5 cursor-pointer group bg-gray-50 dark:bg-gray-950 px-3 py-2 rounded-xl border border-transparent hover:border-gray-200 dark:hover:border-gray-800 transition-all">
                    <input
                      type="checkbox"
                      checked={resumeMode}
                      onChange={(e) => setResumeMode(e.target.checked)}
                      className="w-4 h-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500"
                    />
                    <span className="text-xs font-bold text-gray-500 dark:text-gray-400 group-hover:text-black dark:group-hover:text-white uppercase tracking-tight">
                      Resume (-r) 恢复会话
                    </span>
                  </label>
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-0.5">预览命令</label>
                <div className="bg-gray-50 dark:bg-gray-950 p-4 rounded-xl border border-gray-100 dark:border-gray-800 font-mono text-[11px] text-blue-600 dark:text-blue-400 break-all h-full min-h-[60px]">
                  {generatePreview()}
                </div>
              </div>
            </div>
          </div>

          <div className="pt-4 border-t border-gray-50 dark:border-gray-800 flex justify-center">
            <button
              onClick={() => launchMutation.mutate()}
              disabled={selectedTools.length === 0 || !workDir || launchMutation.isPending}
              className={cn(
                "flex items-center space-x-2 px-10 py-3 rounded-2xl text-base font-black transition-all shadow-lg uppercase tracking-widest",
                selectedTools.length > 0 && workDir && !launchMutation.isPending
                  ? "bg-blue-600 hover:bg-blue-700 text-white shadow-blue-200 dark:shadow-none scale-100 active:scale-95"
                  : "bg-gray-100 dark:bg-gray-800 text-gray-400 cursor-not-allowed shadow-none"
              )}
            >
              {launchMutation.isPending ? (
                <Loader2 className="animate-spin" size={20} />
              ) : (
                <Play size={20} fill="currentColor" />
              )}
              <span>{launchMutation.isPending ? '启动中...' : '启动会话'}</span>
            </button>
          </div>
        </section>

        {/* Right: Recent Sessions */}
        <section className="lg:col-span-2 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl p-6 shadow-sm flex flex-col gap-4 min-h-0 overflow-hidden">
          <div className="flex items-center space-x-2 border-b border-gray-50 dark:border-gray-800 pb-4">
            <History size={18} className="text-gray-400" />
            <h2 className="text-base font-black uppercase tracking-widest text-gray-400">最近会话</h2>
          </div>

          <div className="flex-1 overflow-y-auto space-y-3.5 pr-1 scrollbar-hide pb-4">
            {recentSessions?.map((session, idx) => (
              <div 
                key={`${session.work_dir}-${idx}`}
                className="bg-gray-50 dark:bg-gray-950/50 border border-gray-100 dark:border-gray-800 p-3.5 rounded-xl space-y-3 group hover:border-blue-200 dark:hover:border-blue-900 transition-all shadow-sm"
              >
                <div className="space-y-1.5">
                  <div className="flex items-center space-x-2">
                    <FolderOpen size={14} className="text-gray-400 shrink-0" />
                    <span className="text-sm font-black text-gray-800 dark:text-gray-200 truncate">{session.work_dir}</span>
                  </div>
                  <div className="flex items-start justify-between text-[10px] text-gray-400 font-bold uppercase tracking-tight gap-2">
                    <span className="bg-white dark:bg-gray-900 px-2 py-0.5 rounded border border-gray-100 dark:border-gray-800 flex-1 break-words">
                      {session.tools.join(', ')}
                    </span>
                    <span className="opacity-60 whitespace-nowrap pt-0.5">{formatRelativeTime(session.last_used * 1000)}</span>
                  </div>
                </div>

                <div className="flex gap-1.5">
                  <button
                    onClick={() => {
                      setWorkDirInput(session.work_dir);
                      setSelectedTools(session.tools);
                      setAutoMode(session.auto_mode);
                      setResumeMode(session.resume_mode);
                    }}
                    title="设置"
                    className="flex-1 flex items-center justify-center py-2 border border-gray-100 dark:border-gray-800 hover:bg-white dark:hover:bg-gray-800 rounded-xl text-[10px] font-black uppercase tracking-widest transition-colors shadow-sm whitespace-nowrap"
                  >
                    <RefreshCw size={12} className="mr-1 shrink-0" />
                    <span>设置</span>
                  </button>
                  <button
                    onClick={() => killMutation.mutate(session.work_dir)}
                    disabled={killMutation.isPending}
                    title="关闭"
                    className="flex-1 flex items-center justify-center py-2 border border-red-200 dark:border-red-900 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-xl text-[10px] font-black uppercase tracking-widest transition-colors disabled:opacity-50 shadow-sm whitespace-nowrap"
                  >
                    {killMutation.isPending ? <Loader2 className="animate-spin" size={12} /> : <XCircle size={12} className="mr-1 shrink-0" />}
                    <span>关闭</span>
                  </button>
                  <button
                    onClick={() => connectMutation.mutate(session.work_dir)}
                    disabled={connectMutation.isPending}
                    title="连接"
                    className="flex-[1.2] flex items-center justify-center py-2 bg-blue-500 text-white hover:bg-blue-600 rounded-xl text-[10px] font-black uppercase tracking-widest transition-colors disabled:opacity-50 shadow-md shadow-blue-100 dark:shadow-none whitespace-nowrap"
                  >
                    {connectMutation.isPending ? <Loader2 className="animate-spin" size={12} /> : <Plug size={12} className="mr-1 shrink-0" />}
                    <span>连接</span>
                  </button>
                </div>
              </div>
            ))}
            {(!recentSessions || recentSessions.length === 0) && (
              <div className="py-20 text-center text-gray-400 bg-gray-50 dark:bg-gray-950/20 rounded-xl border border-dashed border-gray-200 dark:border-gray-800 text-xs font-black uppercase tracking-widest">
                暂无会话记录
              </div>
            )}
          </div>
        </section>
      </div>
    </div>

  )
}
