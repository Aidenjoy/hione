import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { 
  listSkills, listSkillRepos, addSkillRepo, removeSkillRepo, 
  deleteSkill, toggleSkillForAgent, syncSkillsToTools 
} from '@/lib/tauri'
import { Plus, Trash2, RefreshCw, Loader2, GitBranch, Terminal } from 'lucide-react'
import * as Switch from '@radix-ui/react-switch'

const AGENTS = ['claude', 'gemini', 'opencode', 'codex', 'qwen']

export default function SkillsPage() {
  const queryClient = useQueryClient()
  const [newRepoUrl, setNewRepoUrl] = useState('')

  const { data: skills, isLoading: loadingSkills } = useQuery({ queryKey: ['skills'], queryFn: listSkills })
  const { data: repos, isLoading: loadingRepos } = useQuery({ queryKey: ['skill-repos'], queryFn: listSkillRepos })

  const addRepoMutation = useMutation({
    mutationFn: (url: string) => addSkillRepo(url),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skill-repos'] })
      setNewRepoUrl('')
    }
  })

  const removeRepoMutation = useMutation({
    mutationFn: (id: string) => removeSkillRepo(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skill-repos'] })
  })

  const deleteSkillMutation = useMutation({
    mutationFn: (id: string) => deleteSkill(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] })
  })

  const toggleMutation = useMutation({
    mutationFn: ({ id, agent, enabled }: { id: string, agent: string, enabled: boolean }) => 
      toggleSkillForAgent(id, agent, enabled),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['skills'] })
  })

  const syncMutation = useMutation({
    mutationFn: syncSkillsToTools,
    onSuccess: () => alert('已同步到工具配置')
  })

  if (loadingSkills || loadingRepos) return <div className="flex justify-center p-20 animate-pulse"><Loader2 className="animate-spin" /></div>

  return (
    <div className="w-full flex flex-col gap-6 animate-in fade-in duration-500">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold">
          Skills 技能管理
        </h1>
        <button
          onClick={() => syncMutation.mutate()}
          disabled={syncMutation.isPending}
          className="flex items-center gap-2 bg-blue-600 text-white px-6 py-2 rounded-xl hover:bg-blue-700 transition-all shadow-md font-bold text-sm disabled:opacity-50"
        >
          <RefreshCw size={18} className={syncMutation.isPending ? 'animate-spin' : ''} /> 同步到工具
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left: Repositories */}
        <div className="lg:col-span-1 flex flex-col gap-4">
          <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-6 shadow-sm flex flex-col gap-6">
            <h2 className="text-xs font-black text-gray-400 uppercase tracking-widest flex items-center gap-2">
              <GitBranch size={14} /> 技能仓库
            </h2>
            
            <div className="flex gap-2">
              <input 
                value={newRepoUrl} onChange={e => setNewRepoUrl(e.target.value)}
                className="flex-1 bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-xl px-4 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="GitHub 仓库 URL"
              />
              <button 
                onClick={() => addRepoMutation.mutate(newRepoUrl)}
                disabled={!newRepoUrl || addRepoMutation.isPending}
                className="bg-gray-100 dark:bg-gray-800 p-2.5 rounded-xl hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
              >
                <Plus size={20} />
              </button>
            </div>

            <div className="flex flex-col gap-3">
              {repos?.map(repo => (
                <div key={repo.id} className="group flex items-center justify-between p-4 border border-gray-50 dark:border-gray-800 rounded-xl bg-gray-50/50 dark:bg-gray-950/50 hover:bg-white dark:hover:bg-gray-900 transition-all border-dashed">
                  <div className="overflow-hidden space-y-0.5">
                    <p className="font-bold text-sm truncate">{repo.name}</p>
                    <p className="text-[10px] text-gray-400 font-mono truncate">{repo.url}</p>
                  </div>
                  <button 
                    onClick={() => removeRepoMutation.mutate(repo.id)}
                    className="opacity-0 group-hover:opacity-100 p-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-all"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              ))}
              {repos?.length === 0 && <p className="text-center py-6 text-xs text-gray-400 italic">暂无仓库</p>}
            </div>
          </div>
        </div>

        {/* Right: Installed Skills */}
        <div className="lg:col-span-2 flex flex-col gap-4">
          <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-6 shadow-sm flex flex-col gap-6">
            <h2 className="text-xs font-black text-gray-400 uppercase tracking-widest flex items-center gap-2">
              <Terminal size={14} /> 已安装技能
            </h2>

            <div className="flex flex-col gap-4">
              {skills?.map(skill => {
                const enabledAgents = Array.isArray(skill.enabled_for) ? skill.enabled_for : JSON.parse(skill.enabled_for as any)
                return (
                  <div key={skill.id} className="border border-gray-50 dark:border-gray-800 rounded-xl p-6 bg-white dark:bg-gray-900 shadow-sm hover:shadow-md transition-shadow flex flex-col gap-6 relative">
                    <div className="flex justify-between items-start">
                      <div className="space-y-1">
                        <h3 className="font-black text-xl">{skill.name}</h3>
                        <p className="text-xs text-gray-400 font-mono">来源: {skill.repo_url || '本地路径'}</p>
                      </div>
                      <button 
                        onClick={() => deleteSkillMutation.mutate(skill.id)}
                        className="text-gray-400 hover:text-red-500 p-2 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                      >
                        <Trash2 size={20} />
                      </button>
                    </div>

                    <div className="flex flex-wrap gap-x-8 gap-y-4 items-center pt-6 border-t border-gray-50 dark:border-gray-800">
                      {AGENTS.map(agent => (
                        <div key={agent} className="flex items-center gap-2">
                          <span className="text-xs font-bold text-gray-600 dark:text-gray-400 capitalize">{agent}</span>
                          <Switch.Root
                            checked={enabledAgents.includes(agent)}
                            onCheckedChange={(checked) => toggleMutation.mutate({ id: skill.id, agent, enabled: checked })}
                            className="w-10 h-5 bg-gray-200 dark:bg-gray-800 rounded-full relative data-[state=checked]:bg-blue-600 outline-none transition-colors"
                          >
                            <Switch.Thumb className="block w-4 h-4 bg-white rounded-full transition-transform translate-x-0.5 data-[state=checked]:translate-x-[1.25rem] shadow-sm" />
                          </Switch.Root>
                        </div>
                      ))}
                    </div>
                  </div>
                )
              })}
              {skills?.length === 0 && (
                <div className="text-center py-20 text-gray-400 italic bg-gray-50/50 dark:bg-gray-950/50 rounded-xl border border-dashed border-gray-200 dark:border-gray-800">
                  暂无已安装的技能
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
