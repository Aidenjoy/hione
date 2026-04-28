import { useState, useEffect, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { listTools, installTool, uninstallTool, checkToolUpdate } from '@/lib/tauri'
import { listen } from '@tauri-apps/api/event'
import { ToolInfo } from '@/types'
import { RefreshCw, CheckCircle2, XCircle, Loader2 } from 'lucide-react'
import { AgentIcon } from '@/components/AgentIcon'

const TOOL_META: Record<string, { icon: string; label: string }> = {
  tmux:     { icon: '🪟', label: 'tmux' },
  claude:   { icon: '🤖', label: 'Claude Code' },
  gemini:   { icon: '💎', label: 'Gemini CLI' },
  opencode: { icon: '🔷', label: 'OpenCode' },
  codex:    { icon: '📦', label: 'Codex CLI' },
  qwen:     { icon: '🌟', label: 'Qwen Code' },
}

function ToolRow({ tool }: { tool: ToolInfo }) {
  const [logs, setLogs] = useState<string[]>([])
  const [showLogs, setShowLogs] = useState(false)
  const [isUpdating, setIsUpdating] = useState(false)
  const logEndRef = useRef<HTMLDivElement>(null)
  const queryClient = useQueryClient()

  const scrollToBottom = () => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [logs])

  useEffect(() => {
    let unlisten: (() => void) | undefined
    const setupListener = async () => {
      const u1 = await listen<{ name: string; line: string }>('tool://install-log', (event) => {
        if (event.payload.name === tool.name) {
          setLogs(prev => [...prev, event.payload.line])
        }
      })
      const u2 = await listen<{ name: string; line: string }>('tool://uninstall-log', (event) => {
        if (event.payload.name === tool.name) {
          setLogs(prev => [...prev, event.payload.line])
        }
      })
      unlisten = () => {
        u1()
        u2()
      }
    }
    setupListener()
    return () => {
      if (unlisten) unlisten()
    }
  }, [tool.name])

  const installMutation = useMutation({
    mutationFn: () => installTool(tool.name),
    onMutate: () => {
      setLogs([`$ 开始安装 ${tool.name}...`])
      setShowLogs(true)
    },
    onSuccess: async () => {
      setLogs(prev => [...prev, '安装成功！'])
      await queryClient.invalidateQueries({ queryKey: ['tools'] })
      await queryClient.refetchQueries({ queryKey: ['tools'] })
    },
    onError: (err: any) => {
      setLogs(prev => [...prev, `错误: ${err}`])
    }
  })

  const uninstallMutation = useMutation({
    mutationFn: () => uninstallTool(tool.name),
    onMutate: () => {
      setLogs([`$ 开始卸载 ${tool.name}...`])
      setShowLogs(true)
    },
    onSuccess: async () => {
      setLogs(prev => [...prev, '卸载成功！'])
      await queryClient.invalidateQueries({ queryKey: ['tools'] })
      await queryClient.refetchQueries({ queryKey: ['tools'] })
    },
    onError: (err: any) => {
      setLogs(prev => [...prev, `错误: ${err}`])
    }
  })

  const handleCheckUpdate = async () => {
    setIsUpdating(true)
    try {
      const newVersion = await checkToolUpdate(tool.name)
      if (newVersion) {
        alert(`发现新版本: ${newVersion}`)
      } else {
        alert('已经是最新版本')
      }
    } catch (err) {
      alert(`检查更新失败: ${err}`)
    } finally {
      setIsUpdating(false)
    }
  }

  const isLoading = installMutation.isPending || uninstallMutation.isPending
  const meta = TOOL_META[tool.name] || { icon: '🛠', label: tool.name }

  return (
    <div className="border-b border-gray-100 dark:border-gray-800 last:border-0 p-6 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="w-10 h-10 flex items-center justify-center">
            <AgentIcon 
              name={tool.name} 
              emoji={meta.icon} 
              size={36} 
            />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <span className="font-bold text-xl">{meta.label}</span>
              {tool.installed ? (
                <span className="flex items-center text-xs text-green-500 bg-green-50 dark:bg-green-900/20 px-2 py-0.5 rounded-full border border-green-200 dark:border-green-800">
                  <CheckCircle2 size={12} className="mr-1" /> 已安装
                </span>
              ) : (
                <span className="flex items-center text-xs text-gray-500 bg-gray-50 dark:bg-gray-800 px-2 py-0.5 rounded-full border border-gray-200 dark:border-gray-700">
                  <XCircle size={12} className="mr-1" /> 未安装
                </span>
              )}
              {tool.version && (
                <span 
                  className="text-sm text-gray-400 font-mono"
                  title={tool.version}
                >
                  v{tool.version.length > 30 ? `${tool.version.slice(0, 30)}...` : tool.version}
                </span>
              )}
            </div>
          </div>
        </div>

        <div className="flex items-center space-x-3">
          {tool.installed ? (
            <>
              <button
                onClick={handleCheckUpdate}
                disabled={isUpdating || isLoading}
                className="text-sm px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-800 transition-all disabled:opacity-50 font-medium"
              >
                {isUpdating ? <Loader2 size={16} className="animate-spin" /> : '检查更新'}
              </button>
              <button
                onClick={() => uninstallMutation.mutate()}
                disabled={isLoading}
                className="text-sm px-4 py-2 border border-red-200 dark:border-red-900 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-xl transition-all disabled:opacity-50 font-medium"
              >
                {uninstallMutation.isPending ? <Loader2 size={16} className="animate-spin" /> : '卸载'}
              </button>
            </>
          ) : (
            <button
              onClick={() => installMutation.mutate()}
              disabled={isLoading}
              className="text-sm px-6 py-2 bg-blue-600 text-white hover:bg-blue-700 rounded-xl shadow-md transition-all disabled:opacity-50 font-bold"
            >
              {installMutation.isPending ? <Loader2 size={16} className="animate-spin" /> : '安装'}
            </button>
          )}
        </div>
      </div>

      {showLogs && (
        <div className="mt-6 animate-in fade-in slide-in-from-top-2 duration-300">
          <button 
            onClick={() => setShowLogs(!showLogs)}
            className="text-xs font-bold text-gray-400 mb-3 flex items-center hover:text-gray-600 uppercase tracking-widest"
          >
            {showLogs ? '▼' : '▶'} 日志面板
          </button>
          <div className="bg-gray-900 text-green-400 text-xs font-mono p-4 rounded-2xl h-40 overflow-y-auto shadow-inner border border-gray-800">
            {logs.map((log, i) => (
              <div key={i} className="whitespace-pre-wrap mb-1">{log}</div>
            ))}
            <div ref={logEndRef} />
          </div>
        </div>
      )}
    </div>
  )
}

export default function ToolManagerPage() {
  const queryClient = useQueryClient()
  const { data: tools, isLoading, isError, error } = useQuery({
    queryKey: ['tools'],
    queryFn: listTools,
    refetchOnMount: 'always',
    staleTime: 0,
  })

  const handleRefresh = async () => {
    await queryClient.invalidateQueries({ queryKey: ['tools'] })
    await queryClient.refetchQueries({ queryKey: ['tools'] })
  }

  if (isLoading) {
    return (
      <div className="flex flex-col gap-6 animate-pulse">
        <h1 className="text-2xl font-bold">工具管理</h1>
        <div className="space-y-4">
          {[1, 2, 3, 4, 5].map(i => (
            <div key={i} className="h-24 bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl" />
          ))}
        </div>
      </div>
    )
  }

  if (isError) {
    return (
      <div className="flex flex-col gap-6">
        <h1 className="text-2xl font-bold">工具管理</h1>
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-6 rounded-xl text-red-600 flex items-center space-x-3 shadow-sm">
          <XCircle />
          <span className="font-bold">加载失败: {String(error)}</span>
        </div>
      </div>
    )
  }

  return (
    <div className="w-full flex flex-col gap-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold">工具管理</h1>
        <button
          onClick={handleRefresh}
          className="flex items-center space-x-2 text-sm px-4 py-2 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-800 transition-all shadow-sm font-bold"
        >
          <RefreshCw size={16} />
          <span>刷新列表</span>
        </button>
      </div>

      <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl overflow-hidden shadow-sm">
        {tools?.map((tool) => (
          <ToolRow key={tool.name} tool={tool} />
        ))}
      </div>
    </div>
  )
}
