import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { readCustomTools, writeCustomTools } from '@/lib/tauri'
import { useAppStore } from '@/lib/store'
import type { CustomTool } from '@/types'
import { Plus, Trash2, Save, X, AlertCircle, Loader2 } from 'lucide-react'

export default function CustomToolsPage() {
  const queryClient = useQueryClient()
  const { currentWorkDir } = useAppStore()
  const [tools, setTools] = useState<CustomTool[]>([])

  const { data: remoteTools, isLoading } = useQuery({
    queryKey: ['custom-tools', currentWorkDir],
    queryFn: () => readCustomTools(currentWorkDir!),
    enabled: !!currentWorkDir
  })

  useEffect(() => {
    if (remoteTools) {
      setTools(remoteTools)
    }
  }, [remoteTools])

  const saveMutation = useMutation({
    mutationFn: (newTools: CustomTool[]) => writeCustomTools(currentWorkDir!, newTools),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['custom-tools', currentWorkDir] })
      alert('保存成功')
    }
  })

  const handleAddTool = () => {
    setTools([...tools, { name: '', auto_flags: [], resume_flags: [] }])
  }

  const handleRemoveTool = (index: number) => {
    setTools(tools.filter((_, i) => i !== index))
  }

  const handleUpdateTool = (index: number, field: keyof CustomTool, value: any) => {
    const newTools = [...tools]
    newTools[index] = { ...newTools[index], [field]: value }
    setTools(newTools)
  }

  const handleAddFlag = (toolIndex: number, type: 'auto_flags' | 'resume_flags') => {
    const newTools = [...tools]
    newTools[toolIndex][type].push('')
    setTools(newTools)
  }

  const handleUpdateFlag = (toolIndex: number, type: 'auto_flags' | 'resume_flags', flagIndex: number, value: string) => {
    const newTools = [...tools]
    newTools[toolIndex][type][flagIndex] = value
    setTools(newTools)
  }

  const handleRemoveFlag = (toolIndex: number, type: 'auto_flags' | 'resume_flags', flagIndex: number) => {
    const newTools = [...tools]
    newTools[toolIndex][type] = newTools[toolIndex][type].filter((_, i) => i !== flagIndex)
    setTools(newTools)
  }

  const generateTomlPreview = () => {
    return tools.map(tool => {
      const name = tool.name || 'unnamed'
      let toml = `[tools.${name}]\n`
      toml += `auto_flags = ${JSON.stringify(tool.auto_flags.filter(f => f))}\n`
      toml += `resume_flags = ${JSON.stringify(tool.resume_flags.filter(f => f))}\n`
      return toml
    }).join('\n')
  }

  if (!currentWorkDir) {
    return (
      <div className="h-full flex flex-col items-center justify-center p-8 text-center gap-4 bg-white dark:bg-gray-900 rounded-xl border border-gray-100 dark:border-gray-800 shadow-sm">
        <div className="bg-yellow-50 dark:bg-yellow-900/20 p-6 rounded-full text-yellow-500">
          <AlertCircle size={48} />
        </div>
        <h2 className="text-xl font-bold">无活跃会话</h2>
        <p className="text-gray-500 max-w-md">请先在「会话启动器」中启动或连接一个 hi 会话，以便编辑当前项目的 tools.toml 配置。</p>
      </div>
    )
  }

  if (isLoading) return <div className="flex justify-center p-20 animate-pulse"><Loader2 className="animate-spin" /></div>

  return (
    <div className="w-full flex flex-col gap-6 animate-in fade-in duration-500">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold">
          自定义工具配置 (tools.toml)
        </h1>
        <div className="flex gap-3">
          <button
            onClick={handleAddTool}
            className="flex items-center gap-2 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 px-4 py-2 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-800 transition-all shadow-sm font-bold text-sm"
          >
            <Plus size={18} /> 添加工具
          </button>
          <button
            onClick={() => saveMutation.mutate(tools)}
            disabled={saveMutation.isPending}
            className="flex items-center gap-2 bg-blue-600 text-white px-6 py-2 rounded-xl hover:bg-blue-700 transition-all shadow-md font-bold text-sm disabled:opacity-50 active:scale-95"
          >
            {saveMutation.isPending ? <Loader2 className="animate-spin" size={18} /> : <Save size={18} />}
            保存
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6 items-start">
        <div className="xl:col-span-2 flex flex-col gap-6">
          {tools.map((tool, toolIdx) => (
            <div key={toolIdx} className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl p-8 flex flex-col gap-8 shadow-sm relative group">
              <button 
                onClick={() => handleRemoveTool(toolIdx)}
                className="absolute top-6 right-6 p-2 text-gray-400 hover:text-red-500 transition-colors"
              >
                <Trash2 size={20} />
              </button>

              <div className="flex flex-col gap-2 max-w-sm">
                <label className="text-xs font-black text-gray-400 uppercase tracking-widest">工具名称</label>
                <input 
                  value={tool.name} onChange={e => handleUpdateTool(toolIdx, 'name', e.target.value)}
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-4 py-3 outline-none focus:ring-2 focus:ring-blue-500 font-black text-lg"
                  placeholder="例如: aider"
                />
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                {/* Auto Flags */}
                <div className="flex flex-col gap-4">
                  <div className="flex justify-between items-center">
                    <label className="text-xs font-black text-gray-400 uppercase tracking-widest">Auto 模式 Flags</label>
                    <button onClick={() => handleAddFlag(toolIdx, 'auto_flags')} className="text-blue-500 hover:text-blue-600 transition-colors p-1">
                      <Plus size={16} />
                    </button>
                  </div>
                  <div className="flex flex-col gap-2">
                    {tool.auto_flags.map((flag, flagIdx) => (
                      <div key={flagIdx} className="flex gap-2 group/flag">
                        <input 
                          value={flag} onChange={e => handleUpdateFlag(toolIdx, 'auto_flags', flagIdx, e.target.value)}
                          className="flex-1 bg-gray-50/50 dark:bg-gray-950 border border-gray-100 dark:border-gray-800 rounded-xl px-3 py-2 text-sm font-mono focus:ring-1 focus:ring-blue-500 outline-none"
                          placeholder="--yes"
                        />
                        <button onClick={() => handleRemoveFlag(toolIdx, 'auto_flags', flagIdx)} className="text-gray-300 hover:text-red-500 transition-colors">
                          <X size={16} />
                        </button>
                      </div>
                    ))}
                    {tool.auto_flags.length === 0 && <p className="text-[10px] text-gray-400 italic">无额外参数</p>}
                  </div>
                </div>

                {/* Resume Flags */}
                <div className="flex flex-col gap-4">
                  <div className="flex justify-between items-center">
                    <label className="text-xs font-black text-gray-400 uppercase tracking-widest">Resume 模式 Flags</label>
                    <button onClick={() => handleAddFlag(toolIdx, 'resume_flags')} className="text-blue-500 hover:text-blue-600 transition-colors p-1">
                      <Plus size={16} />
                    </button>
                  </div>
                  <div className="flex flex-col gap-2">
                    {tool.resume_flags.map((flag, flagIdx) => (
                      <div key={flagIdx} className="flex gap-2 group/flag">
                        <input 
                          value={flag} onChange={e => handleUpdateFlag(toolIdx, 'resume_flags', flagIdx, e.target.value)}
                          className="flex-1 bg-gray-50/50 dark:bg-gray-950 border border-gray-100 dark:border-gray-800 rounded-xl px-3 py-2 text-sm font-mono focus:ring-1 focus:ring-blue-500 outline-none"
                          placeholder="--restore-chat"
                        />
                        <button onClick={() => handleRemoveFlag(toolIdx, 'resume_flags', flagIdx)} className="text-gray-300 hover:text-red-500 transition-colors">
                          <X size={16} />
                        </button>
                      </div>
                    ))}
                    {tool.resume_flags.length === 0 && <p className="text-[10px] text-gray-400 italic">无额外参数</p>}
                  </div>
                </div>
              </div>
            </div>
          ))}
          {tools.length === 0 && (
            <div className="text-center py-24 bg-white dark:bg-gray-900 border border-dashed border-gray-200 dark:border-gray-800 rounded-xl text-gray-400 shadow-sm">
              点击上方按钮添加自定义工具配置
            </div>
          )}
        </div>

        {/* TOML Preview */}
        <div className="xl:col-span-1 sticky top-6">
          <div className="bg-gray-950 rounded-2xl p-6 border border-gray-800 shadow-2xl flex flex-col gap-4">
            <h3 className="text-xs font-black text-gray-500 uppercase tracking-widest">TOML 实时预览</h3>
            <pre className="text-xs font-mono text-green-500 whitespace-pre-wrap overflow-x-auto h-[400px] xl:h-[600px] scrollbar-thin scrollbar-thumb-gray-800 p-2">
              {generateTomlPreview() || '# 暂无内容'}
            </pre>
          </div>
        </div>
      </div>
    </div>
  )
}
