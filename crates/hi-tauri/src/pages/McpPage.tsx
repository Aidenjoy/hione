import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { 
  listMcpServers, createMcpServer, updateMcpServer, 
  deleteMcpServer, toggleMcpForAgent, syncMcpToTools 
} from '@/lib/tauri'
import type { McpServer } from '@/types'
import { Plus, Trash2, Edit2, RefreshCw, Loader2, Terminal } from 'lucide-react'
import * as Dialog from '@radix-ui/react-dialog'
import * as Switch from '@radix-ui/react-switch'

const AGENTS = ['claude', 'gemini', 'opencode', 'codex', 'qwen']

export default function McpPage() {
  const queryClient = useQueryClient()
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [editingServer, setEditingServer] = useState<Partial<McpServer> | null>(null)

  // Form states
  const [name, setName] = useState('')
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState('')
  const [env, setEnv] = useState('')

  const { data: servers, isLoading } = useQuery({
    queryKey: ['mcp'],
    queryFn: listMcpServers
  })

  const createMutation = useMutation({
    mutationFn: (server: McpServer) => createMcpServer(server),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mcp'] })
      setIsDialogOpen(false)
    }
  })

  const updateMutation = useMutation({
    mutationFn: (server: McpServer) => updateMcpServer(server),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mcp'] })
      setIsDialogOpen(false)
    }
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteMcpServer(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['mcp'] })
  })

  const toggleMutation = useMutation({
    mutationFn: ({ id, agent, enabled }: { id: string, agent: string, enabled: boolean }) => 
      toggleMcpForAgent(id, agent, enabled),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['mcp'] })
  })

  const syncMutation = useMutation({
    mutationFn: syncMcpToTools,
    onSuccess: () => alert('已同步到工具配置')
  })

  const handleOpenDialog = (server?: McpServer) => {
    if (server) {
      setEditingServer(server)
      setName(server.name)
      const config = server.server_config as any
      const parsedConfig = typeof config === 'string' ? JSON.parse(config) : config
      setCommand(parsedConfig.command || '')
      setArgs(parsedConfig.args?.join('\n') || '')
      const envLines = Object.entries(parsedConfig.env || {})
        .map(([k, v]) => `${k}=${v}`)
        .join('\n')
      setEnv(envLines)
    } else {
      setEditingServer(null)
      setName('')
      setCommand('')
      setArgs('')
      setEnv('')
    }
    setIsDialogOpen(true)
  }

  const handleSave = () => {
    const envObj: Record<string, string> = {}
    env.split('\n').forEach(line => {
      const [k, v] = line.split('=')
      if (k && v) envObj[k.trim()] = v.trim()
    })

    const server_config = {
      command: command.trim(),
      args: args.split('\n').map(a => a.trim()).filter(a => a),
      env: envObj
    }

    const server: McpServer = {
      id: editingServer?.id || '',
      name: name.trim(),
      server_config: server_config as any,
      enabled_for: editingServer?.enabled_for || "[]" as any
    }

    if (editingServer?.id) {
      updateMutation.mutate(server)
    } else {
      createMutation.mutate(server)
    }
  }

  if (isLoading) return <div className="flex justify-center p-20 animate-pulse"><Loader2 className="animate-spin" /></div>

  return (
    <div className="w-full flex flex-col gap-6 animate-in fade-in duration-500">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold">
          MCP 服务器管理
        </h1>
        <div className="flex gap-3">
          <button
            onClick={() => handleOpenDialog()}
            className="flex items-center gap-2 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 px-4 py-2 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-800 transition-all shadow-sm font-bold text-sm"
          >
            <Plus size={18} /> 添加服务器
          </button>
          <button
            onClick={() => syncMutation.mutate()}
            disabled={syncMutation.isPending}
            className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-xl hover:bg-blue-700 transition-all shadow-md font-bold text-sm disabled:opacity-50"
          >
            <RefreshCw size={18} className={syncMutation.isPending ? 'animate-spin' : ''} /> 同步到配置
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {servers?.map(server => {
          const config = typeof server.server_config === 'string' ? JSON.parse(server.server_config) : server.server_config
          const enabledAgents = Array.isArray(server.enabled_for) ? server.enabled_for : JSON.parse(server.enabled_for as any)
          
          return (
            <div key={server.id} className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-6 flex flex-col gap-6 shadow-sm hover:shadow-md transition-shadow relative group">
              <div className="flex justify-between items-start">
                <div className="space-y-1">
                  <h3 className="text-xl font-black">{server.name}</h3>
                  <div className="text-xs font-mono text-gray-500 flex items-center gap-2 bg-gray-50 dark:bg-gray-950 px-2 py-1 rounded">
                    <Terminal size={12} /> {config.command}
                  </div>
                </div>
                <div className="flex gap-2">
                  <button onClick={() => handleOpenDialog(server)} className="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors text-gray-400 hover:text-blue-500">
                    <Edit2 size={16} />
                  </button>
                  <button onClick={() => deleteMutation.mutate(server.id)} className="p-2 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors text-gray-400 hover:text-red-500">
                    <Trash2 size={16} />
                  </button>
                </div>
              </div>

              <div className="border-t border-gray-50 dark:border-gray-800 pt-6">
                <p className="text-[10px] font-black text-gray-400 uppercase tracking-widest mb-4">启用状态</p>
                <div className="flex flex-wrap gap-x-6 gap-y-4">
                  {AGENTS.map(agent => (
                    <div key={agent} className="flex items-center gap-2">
                      <span className="text-xs font-bold text-gray-600 dark:text-gray-400 capitalize">{agent}</span>
                      <Switch.Root
                        checked={enabledAgents.includes(agent)}
                        onCheckedChange={(checked) => toggleMutation.mutate({ id: server.id, agent, enabled: checked })}
                        className="w-10 h-5 bg-gray-200 dark:bg-gray-800 rounded-full relative data-[state=checked]:bg-blue-600 outline-none transition-colors"
                      >
                        <Switch.Thumb className="block w-4 h-4 bg-white rounded-full transition-transform translate-x-0.5 data-[state=checked]:translate-x-[1.25rem] shadow-sm" />
                      </Switch.Root>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )
        })}
        {servers?.length === 0 && (
          <div className="col-span-full py-20 text-center text-gray-400 bg-white dark:bg-gray-900 rounded-xl border border-dashed border-gray-200 dark:border-gray-800 shadow-sm">
            暂无 MCP 服务器，点击上方按钮添加。
          </div>
        )}
      </div>

      <Dialog.Root open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 animate-in fade-in" />
          <Dialog.Content className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-lg bg-white dark:bg-gray-900 rounded-3xl p-8 shadow-2xl z-50 animate-in zoom-in-95 duration-200 border border-gray-100 dark:border-gray-800">
            <Dialog.Title className="text-2xl font-black mb-6">
              {editingServer ? '编辑 MCP 服务器' : '添加 MCP 服务器'}
            </Dialog.Title>
            
            <div className="flex flex-col gap-6">
              <div className="flex flex-col gap-2">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">服务器名称</label>
                <input 
                  value={name} onChange={e => setName(e.target.value)}
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-4 py-3 outline-none focus:ring-2 focus:ring-blue-500 font-bold" 
                  placeholder="例如: Google Maps"
                />
              </div>

              <div className="flex flex-col gap-2">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">执行命令 (Command)</label>
                <input 
                  value={command} onChange={e => setCommand(e.target.value)}
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-4 py-3 font-mono outline-none focus:ring-2 focus:ring-blue-500 text-sm" 
                  placeholder="npx"
                />
              </div>

              <div className="flex flex-col gap-2">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">参数 (Args - 每行一个)</label>
                <textarea 
                  value={args} onChange={e => setArgs(e.target.value)}
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-4 py-3 font-mono h-24 outline-none focus:ring-2 focus:ring-blue-500 text-sm resize-none"
                  placeholder="-y&#10;@modelcontextprotocol/server-google-maps"
                />
              </div>

              <div className="flex flex-col gap-2">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">环境变量 (Env - key=value 每行一个)</label>
                <textarea 
                  value={env} onChange={e => setEnv(e.target.value)}
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-4 py-3 font-mono h-24 outline-none focus:ring-2 focus:ring-blue-500 text-sm resize-none"
                  placeholder="GOOGLE_MAPS_API_KEY=xxx"
                />
              </div>
            </div>

            <div className="flex justify-end gap-3 mt-10">
              <Dialog.Close className="px-6 py-2.5 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-2xl transition-colors font-bold text-sm">取消</Dialog.Close>
              <button 
                onClick={handleSave}
                disabled={!name || !command || createMutation.isPending || updateMutation.isPending}
                className="bg-blue-600 text-white px-8 py-2.5 rounded-2xl font-black hover:bg-blue-700 disabled:opacity-50 transition-all shadow-lg shadow-blue-200 dark:shadow-none flex items-center gap-2 active:scale-95"
              >
                {(createMutation.isPending || updateMutation.isPending) && <Loader2 className="animate-spin" size={16} />}
                保存
              </button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  )
}
