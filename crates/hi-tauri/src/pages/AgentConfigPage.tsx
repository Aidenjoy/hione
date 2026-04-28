import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { listAgents, saveAgent, testAgentConnection } from '@/lib/tauri'
import type { Agent } from '@/types'
import { Eye, EyeOff, Link as LinkIcon, Save, Loader2, CheckCircle, XCircle } from 'lucide-react'
import { cn } from '@/lib/utils'
import { AgentIcon } from '@/components/AgentIcon'

const TOOL_META: Record<string, { icon: string; label: string; defaultBaseUrl: string }> = {
  claude:   { icon: '🤖', label: 'Claude Code',  defaultBaseUrl: 'https://api.anthropic.com' },
  gemini:   { icon: '💎', label: 'Gemini CLI',   defaultBaseUrl: 'https://generativelanguage.googleapis.com' },
  opencode: { icon: '🔷', label: 'OpenCode',     defaultBaseUrl: 'https://api.openai.com' },
  codex:    { icon: '📦', label: 'Codex CLI',    defaultBaseUrl: 'https://api.openai.com' },
  qwen:     { icon: '🌟', label: 'Qwen Code',    defaultBaseUrl: 'https://dashscope.aliyuncs.com' },
}

export default function AgentConfigPage() {
  const queryClient = useQueryClient()
  const [selectedAgentName, setSelectedAgentName] = useState<string>('claude')
  const [showPassword, setShowPassword] = useState(false)
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null)
  const [saveSuccess, setSaveSuccess] = useState(false)

  // Local form state
  const [formData, setFormData] = useState<Partial<Agent>>({
    api_key: '',
    api_base_url: '',
    model: '',
  })

  const { data: agents, isLoading } = useQuery({
    queryKey: ['agents'],
    queryFn: listAgents,
  })

  const currentAgent = agents?.find(a => a.name === selectedAgentName)

  useEffect(() => {
    if (currentAgent) {
      setFormData({
        api_key: currentAgent.api_key || '',
        api_base_url: currentAgent.api_base_url || '',
        model: currentAgent.model || '',
      })
      setTestResult(null)
      setSaveSuccess(false)
    }
  }, [currentAgent])

  const saveMutation = useMutation({
    mutationFn: (agent: Agent) => saveAgent(agent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] })
      setSaveSuccess(true)
      setTimeout(() => setSaveSuccess(false), 3000)
    },
  })

  const testMutation = useMutation({
    mutationFn: (name: string) => testAgentConnection(name),
    onSuccess: (success) => {
      setTestResult({
        success,
        message: success ? '连接成功' : '连接失败',
      })
      setTimeout(() => setTestResult(null), 3000)
    },
    onError: (err: any) => {
      setTestResult({
        success: false,
        message: `测试出错: ${err}`,
      })
      setTimeout(() => setTestResult(null), 3000)
    },
  })

  const handleSave = () => {
    if (!currentAgent) return
    const updatedAgent: Agent = {
      ...currentAgent,
      api_key: formData.api_key,
      api_base_url: formData.api_base_url,
      model: formData.model,
    }
    saveMutation.mutate(updatedAgent)
  }

  const handleTest = () => {
    testMutation.mutate(selectedAgentName)
  }

  if (isLoading) {
    return (
      <div className="flex flex-col gap-6 animate-pulse">
        <h1 className="text-2xl font-bold">Agent 配置</h1>
        <div className="h-[500px] bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl" />
      </div>
    )
  }

  return (
    <div className="w-full flex flex-col gap-6 h-full overflow-hidden">
      <h1 className="text-2xl font-bold">
        Agent 配置
      </h1>

      <div className="flex-1 flex bg-white dark:bg-gray-900 overflow-hidden rounded-xl border border-gray-100 dark:border-gray-800 shadow-sm min-h-0">
        {/* Left Sidebar */}
        <div className="w-44 border-r border-gray-50 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-950/50 overflow-y-auto">
          <div className="p-4 text-xs font-black text-gray-400 uppercase tracking-widest">工具列表</div>
          <nav className="px-2 space-y-1">
            {Object.entries(TOOL_META).map(([name, meta]) => (
              <button
                key={name}
                onClick={() => setSelectedAgentName(name)}
                className={cn(
                  "w-full flex items-center space-x-3 px-4 py-4 rounded-xl text-sm transition-all",
                  selectedAgentName === name
                    ? "bg-blue-600 text-white shadow-lg shadow-blue-100 dark:shadow-none font-bold scale-[1.02]"
                    : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
                )}
              >
                <AgentIcon name={name} emoji={meta.icon} size={20} />
                <span className="truncate">{meta.label}</span>
              </button>
            ))}
          </nav>
        </div>

        {/* Right Content */}
        <div className="flex-1 p-10 overflow-y-auto">
          <div className="max-w-2xl space-y-10">
            <header className="space-y-2 pb-6 border-b border-gray-50 dark:border-gray-800">
              <h2 className="text-3xl font-black flex items-center space-x-4">
                <AgentIcon name={selectedAgentName} emoji={TOOL_META[selectedAgentName].icon} size={32} />
                <span>{TOOL_META[selectedAgentName].label}</span>
              </h2>
              <p className="text-gray-500 font-medium">配置您的 API 密钥、模型版本及基础地址</p>
            </header>

            <div className="space-y-8">
              {/* API Key */}
              <div className="space-y-3">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">API Key</label>
                <div className="relative">
                  <input
                    type={showPassword ? "text" : "password"}
                    value={formData.api_key}
                    onChange={(e) => setFormData({ ...formData, api_key: e.target.value })}
                    placeholder="在此输入您的 API 密钥"
                    className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-5 py-4 outline-none focus:ring-2 focus:ring-blue-500 transition-all pr-14 font-mono"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 transition-colors"
                  >
                    {showPassword ? <EyeOff size={20} /> : <Eye size={20} />}
                  </button>
                </div>
              </div>

              {/* Base URL */}
              <div className="space-y-3">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">Base URL</label>
                <input
                  type="text"
                  value={formData.api_base_url}
                  onChange={(e) => setFormData({ ...formData, api_base_url: e.target.value })}
                  placeholder={TOOL_META[selectedAgentName].defaultBaseUrl}
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-5 py-4 outline-none focus:ring-2 focus:ring-blue-500 transition-all font-mono"
                />
              </div>

              {/* Model */}
              <div className="space-y-3">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">Model Name</label>
                <input
                  type="text"
                  value={formData.model}
                  onChange={(e) => setFormData({ ...formData, model: e.target.value })}
                  placeholder="例如: claude-3-5-sonnet-latest"
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-5 py-4 outline-none focus:ring-2 focus:ring-blue-500 transition-all font-mono"
                />
              </div>

              {/* Actions */}
              <div className="pt-6 flex items-center space-x-4 border-t border-gray-50 dark:border-gray-800">
                <button
                  onClick={handleTest}
                  disabled={testMutation.isPending}
                  className="flex items-center space-x-2 px-6 py-3 border border-gray-200 dark:border-gray-800 rounded-2xl text-sm font-bold hover:bg-gray-50 dark:hover:bg-gray-800 transition-all disabled:opacity-50"
                >
                  {testMutation.isPending ? (
                    <Loader2 className="animate-spin" size={18} />
                  ) : (
                    <LinkIcon size={18} />
                  )}
                  <span>测试连接</span>
                </button>

                <button
                  onClick={handleSave}
                  disabled={saveMutation.isPending}
                  className="flex items-center space-x-2 px-8 py-3 bg-blue-600 text-white rounded-2xl text-sm font-black hover:bg-blue-700 transition-all disabled:opacity-50 shadow-lg shadow-blue-100 dark:shadow-none active:scale-95"
                >
                  {saveMutation.isPending ? (
                    <Loader2 className="animate-spin" size={18} />
                  ) : (
                    <Save size={18} />
                  )}
                  <span>保存设置</span>
                </button>

                {testResult && (
                  <div className={cn(
                    "flex items-center space-x-2 px-4 py-2 rounded-full text-xs font-bold animate-in fade-in slide-in-from-left-4 duration-300",
                    testResult.success 
                      ? "bg-green-50 text-green-600 border border-green-100"
                      : "bg-red-50 text-red-600 border border-red-100"
                  )}>
                    {testResult.success ? <CheckCircle size={16} /> : <XCircle size={16} />}
                    <span>{testResult.message}</span>
                  </div>
                )}

                {saveSuccess && (
                  <div className="flex items-center space-x-2 px-4 py-2 rounded-full text-xs font-bold bg-green-50 text-green-600 border border-green-100 animate-in fade-in slide-in-from-left-4 duration-300">
                    <CheckCircle size={16} />
                    <span>设置已保存</span>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
