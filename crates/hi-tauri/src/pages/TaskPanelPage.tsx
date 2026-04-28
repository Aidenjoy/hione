import { useState, useEffect, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { listTasks, pushTask, cancelTask, checkAgent } from '@/lib/tauri'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '@/lib/store'
import type { TaskRecord } from '@/types'
import { cn } from '@/lib/utils'
import { StatusIcon } from '@/components/StatusIcon'
import { 
  CheckCircle2, 
  XCircle, 
  Clock, 
  ChevronDown, 
  ChevronUp, 
  Send, 
  RefreshCw, 
  AlertCircle,
  Loader2,
  RotateCcw,
  Plug
} from 'lucide-react'

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

const AGENT_LIST = ['claude', 'opencode', 'gemini', 'codex', 'qwen'];

function TaskHistoryItem({ task, onRetry }: { task: TaskRecord; onRetry: (t: TaskRecord) => void }) {
  const [expanded, setExpanded] = useState(false);
  const queryClient = useQueryClient();

  const cancelMutation = useMutation({
    mutationFn: (id: string) => cancelTask(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    }
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'Completed':
        return <CheckCircle2 size={16} className="text-green-500" />;
      case 'Timeout':
      case 'Failed':
        return <XCircle size={16} className="text-red-500" />;
      default:
        return <Clock size={16} className="text-blue-500 animate-pulse" />;
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'Completed': return '完成';
      case 'Timeout': return '超时';
      case 'Failed': return '失败';
      case 'Pending': return '进行中';
      case 'Running': return '执行中';
      default: return status;
    }
  };

  return (
    <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-3 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors group shadow-sm">
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-2.5 w-full">
          <div className="mt-0.5 shrink-0">{getStatusIcon(task.status)}</div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center space-x-2 text-[9px] text-gray-400 mb-0.5">
              <span className="font-mono bg-gray-100 dark:bg-gray-800 px-1 rounded truncate">[{task.id.slice(0, 7)}]</span>
              <span className="font-bold uppercase">→ {task.receiver}</span>
            </div>
            <p className="text-xs font-bold text-gray-800 dark:text-gray-200 line-clamp-2 mb-1.5 break-all leading-relaxed">
              {task.content}
            </p>
            <div className="flex items-center space-x-2 text-[9px] text-gray-500 font-medium">
              <span className="bg-gray-50 dark:bg-gray-800 px-1.5 py-0.5 rounded-full">{getStatusText(task.status)}</span>
              <span>·</span>
              <span>{formatRelativeTime(task.created_at * 1000)}</span>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-3 flex items-center justify-between border-t border-gray-50 dark:border-gray-800 pt-2">
        <div className="flex items-center space-x-2">
          {task.status !== 'Completed' && task.status !== 'Timeout' && task.status !== 'Failed' && (
            <button
              onClick={() => cancelMutation.mutate(task.id)}
              disabled={cancelMutation.isPending}
              className="text-[10px] text-red-500 hover:text-red-600 font-black uppercase tracking-widest px-2 py-0.5 rounded hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors disabled:opacity-50"
            >
              取消
            </button>
          )}
          {(task.status === 'Timeout' || task.status === 'Failed') && (
            <button
              onClick={() => onRetry(task)}
              className="flex items-center space-x-1 text-[10px] text-blue-500 hover:text-blue-600 font-black uppercase tracking-widest px-2 py-0.5 rounded hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors"
            >
              <RotateCcw size={10} />
              <span>重试</span>
            </button>
          )}
        </div>

        {task.status === 'Completed' && (
          <button
            onClick={() => setExpanded(!expanded)}
            className="flex items-center space-x-1 text-[9px] text-gray-400 hover:text-gray-600 font-black uppercase tracking-widest transition-colors"
          >
            <span>{expanded ? '收起' : '详情'}</span>
            {expanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
          </button>
        )}
      </div>

      {expanded && (
        <div className="mt-2 p-2.5 bg-gray-50 dark:bg-gray-950 rounded-lg border border-gray-100 dark:border-gray-800 animate-in fade-in slide-in-from-top-1 duration-200">
          <pre className="text-[10px] font-mono text-gray-500 dark:text-gray-400 whitespace-pre-wrap break-all leading-tight">
            {(task as any).result || '无结果内容'}
          </pre>
        </div>
      )}
    </div>
  );
}

export default function TaskPanelPage() {
  const queryClient = useQueryClient();
  const { sessionConnected } = useAppStore();
  
  const [target, setTarget] = useState('claude');
  const [content, setContent] = useState('');
  const [agentStatuses, setAgentStatuses] = useState<Record<string, boolean>>({});
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const { data: tasks, isLoading: tasksLoading } = useQuery({
    queryKey: ['tasks'],
    queryFn: listTasks,
    refetchInterval: 5000,
    enabled: sessionConnected
  });

  const sortedTasks = [...(tasks || [])].sort((a, b) => b.created_at - a.created_at);

  const pushMutation = useMutation({
    mutationFn: ({ target, content }: { target: string; content: string }) => pushTask(target, content),
    onSuccess: () => {
      setContent('');
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    }
  });

  // Listen for task updates
  useEffect(() => {
    const unCompleted = listen<{ task_id: string; result: string }>('task://completed', () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    });
    const unTimeout = listen<{ task_id: string }>('task://timeout', () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    });
    
    return () => {
      unCompleted.then(f => f());
      unTimeout.then(f => f());
    };
  }, [queryClient]);

  // Poll agent statuses
  useEffect(() => {
    const checkAllAgents = async () => {
      const statuses: Record<string, boolean> = {};
      await Promise.all(AGENT_LIST.map(async (name) => {
        try {
          statuses[name] = await checkAgent(name);
        } catch {
          statuses[name] = false;
        }
      }));
      setAgentStatuses(statuses);
    };

    checkAllAgents();
    const interval = setInterval(checkAllAgents, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleSend = () => {
    if (!content.trim() || !sessionConnected || pushMutation.isPending) return;
    pushMutation.mutate({ target, content });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleRetry = (task: TaskRecord) => {
    setTarget(task.receiver);
    setContent(task.content);
    textareaRef.current?.focus();
  };

  if (!sessionConnected) {
    return (
      <div className="h-full flex flex-col items-center justify-center p-8 text-center gap-4 bg-white dark:bg-gray-900 rounded-xl border border-gray-100 dark:border-gray-800 shadow-sm animate-in fade-in duration-500">
        <div className="bg-blue-50 dark:bg-blue-900/20 p-6 rounded-full">
          <Plug size={40} className="text-blue-500" />
        </div>
        <h2 className="text-xl font-black">未连接到会话</h2>
        <p className="text-xs text-gray-500 max-w-xs font-medium leading-relaxed">
          请先在「会话启动器」中启动或连接一个 hi 会话，然后才能管理任务。
        </p>
      </div>
    );
  }

  return (
    <div className="w-full flex flex-col gap-6 h-full overflow-hidden animate-in fade-in duration-500">
      <h1 className="text-2xl font-black tracking-tight">任务面板</h1>

      <div className="flex-1 flex flex-col md:flex-row gap-6 min-h-0">
        {/* Left Column: Task History */}
        <div className="w-full md:w-[380px] flex flex-col gap-4">
          <div className="flex items-center justify-between px-1">
            <h2 className="text-xs font-black text-gray-400 uppercase tracking-widest flex items-center gap-2">
              <RefreshCw size={14} className={cn(tasksLoading && "animate-spin")} />
              <span>任务历史</span>
            </h2>
            <span className="text-[10px] font-black text-gray-400 bg-gray-100 dark:bg-gray-800 px-2 py-0.5 rounded-full">{tasks?.length || 0}</span>
          </div>

          <div className="flex-1 overflow-y-auto pr-2 flex flex-col gap-3 pb-4 scrollbar-hide">
            {sortedTasks.map(task => (
              <TaskHistoryItem key={task.id} task={task} onRetry={handleRetry} />
            ))}
            {sortedTasks.length === 0 && (
              <div className="py-20 text-center text-xs font-black text-gray-400 bg-white dark:bg-gray-900 rounded-xl border border-dashed border-gray-200 dark:border-gray-800 shadow-sm">
                暂无任务记录
              </div>
            )}
          </div>
        </div>

        {/* Right Column: Send New Task */}
        <div className="flex-1 flex flex-col gap-6 bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-8 shadow-sm min-h-0 overflow-y-auto scrollbar-hide">
          <div className="flex flex-col gap-6">
            <h2 className="text-sm font-black flex items-center gap-2 text-gray-400 uppercase tracking-widest">
              <Send size={18} className="text-blue-500" />
              <span>发送新任务</span>
            </h2>

            <div className="flex flex-col gap-4">
              <div className="flex flex-col gap-2">
                <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-0.5">发送给</label>
                <select
                  value={target}
                  onChange={(e) => setTarget(e.target.value)}
                  className="w-full bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-xl px-4 py-2.5 outline-none focus:ring-2 focus:ring-blue-500 transition-all appearance-none cursor-pointer text-sm font-bold"
                >
                  {AGENT_LIST.map(name => (
                    <option key={name} value={name}>{name}</option>
                  ))}
                </select>
              </div>

              <div className="flex flex-col gap-2">
                <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-0.5">任务内容</label>
                <textarea
                  ref={textareaRef}
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="在此输入任务详情... (Ctrl+Enter 发送)"
                  className="w-full h-40 bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-xl px-4 py-4 outline-none focus:ring-2 focus:ring-blue-500 transition-all resize-none text-sm font-medium"
                />
              </div>
            </div>

            <div className="flex items-center justify-between pt-4 border-t border-gray-50 dark:border-gray-800">
              <div className="flex flex-wrap gap-3">
                {AGENT_LIST.map(name => (
                  <div key={name} className="flex items-center gap-1.5 px-2 py-1 bg-gray-50 dark:bg-gray-800 rounded-lg text-xs">
                    <StatusIcon status={agentStatuses[name] ? 'online' : 'offline'} size={10} />
                    <span className="text-gray-500 dark:text-gray-400 font-bold">{name}</span>
                  </div>
                ))}
              </div>

              <button
                onClick={handleSend}
                disabled={!content.trim() || pushMutation.isPending}
                className={cn(
                  "flex items-center gap-2 px-8 py-3 rounded-xl font-bold transition-all shadow-md active:scale-95 text-sm",
                  content.trim() && !pushMutation.isPending
                    ? "bg-blue-600 hover:bg-blue-700 text-white shadow-blue-200 dark:shadow-none"
                    : "bg-gray-100 dark:bg-gray-800 text-gray-400 cursor-not-allowed shadow-none"
                )}
              >
                {pushMutation.isPending ? (
                  <Loader2 className="animate-spin" size={18} />
                ) : (
                  <Send size={18} />
                )}
                <span>发送任务</span>
              </button>
            </div>
          </div>

          <div className="bg-blue-50 dark:bg-blue-900/10 p-4 rounded-xl flex items-start gap-3 border border-blue-100 dark:border-blue-900/30">
            <AlertCircle size={18} className="text-blue-500 shrink-0 mt-0.5" />
            <div className="text-xs text-blue-700 dark:text-blue-400 leading-relaxed font-medium">
              <p className="font-black mb-1 uppercase tracking-widest text-[10px]">提示</p>
              <ul className="list-disc list-inside space-y-1">
                <li>Shift+Enter 换行，Ctrl+Enter 发送</li>
                <li>任务发送后将进入队列，Agent 空闲时会自动执行</li>
                <li>超时任务可以通过左侧历史面板进行重试</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>

  );
}
