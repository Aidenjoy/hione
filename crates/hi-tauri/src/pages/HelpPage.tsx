import { useState } from 'react'
import { Terminal, Copy, Check, ChevronRight } from 'lucide-react'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { useTranslation } from 'react-i18next'

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

const COMMANDS = [
  {
    name: 'hi start',
    alias: 'hi s',
    desc: '启动多窗口 AI 协作会话',
    usage: 'hi start [-a] [-r] <tools>',
    flags: [
      { flag: '-a, --auto', desc: '跳过所有权限确认提示' },
      { flag: '-r, --resume', desc: '恢复上次会话上下文' },
    ],
    examples: ['hi start claude,opencode,gemini', 'hi s -a -r claude,opencode'],
  },
  { 
    name: 'hi push', 
    alias: 'hi p', 
    desc: '向目标 Agent 派发任务', 
    usage: 'hi push <target> "<task>"', 
    flags: [], 
    examples: ['hi push opencode "实现登录接口"', 'hi p gemini "修复样式"'] 
  },
  { 
    name: 'hi pull', 
    alias: 'hi pl', 
    desc: '拉取目标窗口当前内容', 
    usage: 'hi pull <target> [-t N]', 
    flags: [{ flag: '-t, --timeout', desc: '超时秒数，默认 5' }], 
    examples: ['hi pull opencode', 'hi pl gemini -t 10'] 
  },
  { 
    name: 'hi check', 
    alias: 'hi ck', 
    desc: '检查 Agent 在线状态', 
    usage: 'hi check <target>', 
    flags: [], 
    examples: ['hi check opencode'] 
  },
  { 
    name: 'hi result', 
    alias: 'hi r', 
    desc: '回复任务结果', 
    usage: 'hi result <id> "<content>"', 
    flags: [], 
    examples: ['hi r <uuid> "完成"'] 
  },
  { 
    name: 'hi esc', 
    alias: 'hi e', 
    desc: '取消任务', 
    usage: 'hi esc <id>', 
    flags: [], 
    examples: ['hi e <uuid>'] 
  },
]

export default function HelpPage() {
  const { t } = useTranslation()
  const [activeCmd, setActiveCmd] = useState(COMMANDS[0])
  const [copied, setCopied] = useState<string | null>(null)

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text)
    setCopied(text)
    setTimeout(() => setCopied(null), 2000)
  }

  return (
    <div className="w-full flex flex-col gap-4 h-full overflow-hidden">
      <h1 className="text-xl font-black tracking-tight">
        {t('nav.help')}
      </h1>

      <div className="flex-1 flex bg-white dark:bg-gray-900 overflow-hidden rounded-xl border border-gray-100 dark:border-gray-800 shadow-sm min-h-0">
        {/* Left Sidebar: Command List */}
        <div className="w-56 border-r border-gray-50 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-950/50 overflow-y-auto">
          <div className="p-4 border-b border-gray-100 dark:border-gray-800">
            <h2 className="text-[10px] font-black text-gray-400 uppercase tracking-widest flex items-center space-x-2">
              <Terminal size={12} />
              <span>{t('help.cliRef')}</span>
            </h2>
          </div>
          <nav className="p-2 space-y-0.5">
            {COMMANDS.map((cmd) => (
              <button
                key={cmd.name}
                type="button"
                onClick={() => setActiveCmd(cmd)}
                className={cn(
                  "w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-sm transition-all text-left",
                  activeCmd.name === cmd.name
                    ? "bg-blue-600 text-white shadow-lg shadow-blue-100 dark:shadow-none font-bold scale-[1.02]"
                    : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
                )}
              >
                <div className="flex flex-col items-start min-w-0">
                  <span className="truncate w-full font-bold">{cmd.name}</span>
                  <span className={cn("text-[9px] opacity-60 font-normal", activeCmd.name === cmd.name ? "text-blue-100" : "text-gray-400")}>
                    别名: {cmd.alias}
                  </span>
                </div>
                <ChevronRight size={12} className={cn("shrink-0 transition-opacity", activeCmd.name === cmd.name ? "opacity-100" : "opacity-0")} />
              </button>
            ))}
          </nav>
        </div>

        {/* Right Content: Command Details */}
        <div className="flex-1 overflow-y-auto p-6 scrollbar-hide">
          <div className="max-w-3xl space-y-6">
            <header className="space-y-2 pb-4 border-b border-gray-50 dark:border-gray-800">
              <div className="flex items-center space-x-2">
                <h1 className="text-2xl font-black tracking-tight">{activeCmd.name}</h1>
                <span className="bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 text-[10px] px-2 py-0.5 rounded-full font-black border border-blue-100 dark:border-blue-900 shrink-0">
                  {activeCmd.alias}
                </span>
              </div>
              <p className="text-sm text-gray-500 font-medium leading-relaxed">{activeCmd.desc}</p>
            </header>

            <section className="space-y-2">
              <h3 className="text-[9px] font-black text-gray-400 uppercase tracking-widest">使用方法</h3>
              <div className="bg-gray-50 dark:bg-gray-950 p-3 rounded-xl border border-gray-100 dark:border-gray-800 font-mono text-blue-600 dark:text-blue-400 text-base flex items-center justify-between group shadow-inner">
                <code className="break-all">{activeCmd.usage}</code>
                <button onClick={() => handleCopy(activeCmd.usage)} className="opacity-0 group-hover:opacity-100 p-1 hover:bg-white dark:hover:bg-gray-800 rounded-lg transition-all text-gray-400 shrink-0 ml-2">
                  {copied === activeCmd.usage ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
                </button>
              </div>
            </section>

            {activeCmd.flags.length > 0 && (
              <section className="space-y-2">
                <h3 className="text-[9px] font-black text-gray-400 uppercase tracking-widest">参数说明</h3>
                <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl overflow-hidden shadow-sm">
                  <table className="w-full text-left border-collapse">
                    <thead>
                      <tr className="bg-gray-50 dark:bg-gray-800/50">
                        <th className="px-3 py-1.5 text-[9px] font-black text-gray-400 uppercase tracking-widest">参数 (Flag)</th>
                        <th className="px-3 py-1.5 text-[9px] font-black text-gray-400 uppercase tracking-widest">说明 (Description)</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-50 dark:divide-gray-800">
                      {activeCmd.flags.map((f, i) => (
                        <tr key={i} className="hover:bg-gray-50/30 dark:hover:bg-gray-800/30 transition-colors">
                          <td className="px-3 py-2 font-mono text-[10px] text-blue-600 dark:text-blue-400 font-bold whitespace-nowrap">{f.flag}</td>
                          <td className="px-3 py-2 text-[10px] text-gray-600 dark:text-gray-400">{f.desc}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </section>
            )}

            <section className="space-y-3 pt-1">
              <h3 className="text-[9px] font-black text-gray-400 uppercase tracking-widest">示例代码</h3>
              <div className="flex flex-col gap-1.5">
                {activeCmd.examples.map((ex, i) => (
                  <div key={i} className="bg-gray-950 rounded-lg p-3 border border-gray-800 flex items-center justify-between group shadow-lg">
                    <code className="font-mono text-green-400 text-[11px] flex items-center break-all">
                      <span className="text-gray-600 mr-2 select-none shrink-0">$</span>
                      {ex}
                    </code>
                    <button onClick={() => handleCopy(ex)} className="opacity-0 group-hover:opacity-100 p-1 hover:bg-gray-800 rounded-lg transition-all text-gray-400 shrink-0 ml-2">
                      {copied === ex ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
                    </button>
                  </div>
                ))}
              </div>
            </section>
          </div>
        </div>
      </div>
    </div>



  )
}
