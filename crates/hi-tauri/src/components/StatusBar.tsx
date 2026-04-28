import React, { useEffect, useState, useMemo } from 'react';
import { useAppStore } from '@/lib/store';
import { StatusIcon } from './StatusIcon';
import { Terminal, HardDrive } from 'lucide-react';
import { checkAgent } from '@/lib/tauri';
import { AgentIcon } from './AgentIcon';

// Agent emoji mapping
const AGENT_EMOJIS: Record<string, string> = {
  claude: '🤖',
  opencode: '🔷',
  gemini: '💎',
  codex: '📜',
  qwen: '🌟',
  ccg: '🧠',
};

export const StatusBar: React.FC = () => {
  const { sessionConnected, currentWorkDir, sessionInfo } = useAppStore();
  const [agentStatus, setAgentStatus] = useState<Record<string, boolean>>({});

  // Memoize agents array to prevent infinite re-renders
  const agents = useMemo(() =>
    sessionInfo?.windows?.map(w => ({
      name: w.name,
      emoji: AGENT_EMOJIS[w.name] || '❓',
    })) || [], [sessionInfo?.windows]);

  useEffect(() => {
    if (!sessionConnected || !currentWorkDir || agents.length === 0) {
      setAgentStatus({});
      return;
    }

    const checkAll = async () => {
      const status: Record<string, boolean> = {};
      for (const agent of agents) {
        try {
          status[agent.name] = await checkAgent(agent.name);
        } catch {
          status[agent.name] = false;
        }
      }
      setAgentStatus(status);
    };

    checkAll();
    const interval = setInterval(checkAll, 10000);
    return () => clearInterval(interval);
  }, [sessionConnected, currentWorkDir, agents]);

  return (
    <div className="h-8 bg-white dark:bg-gray-900 border-t border-gray-100 dark:border-gray-800 flex items-center justify-between px-4 text-[10px] font-medium text-gray-500 select-none">
      <div className="flex items-center space-x-6">
        <div className="flex items-center space-x-1.5">
          <StatusIcon status={sessionConnected ? 'online' : 'offline'} size={8} />
          <span className="font-bold">{sessionConnected ? '会话已连接' : '未连接'}</span>
        </div>

        {sessionConnected && agents.length > 0 && (
          <div className="flex items-center space-x-4 border-l border-gray-100 dark:border-gray-800 pl-4">
            {agents.map(agent => (
              <div key={agent.name} className="flex items-center space-x-1.5 grayscale-[0.5] hover:grayscale-0 transition-all cursor-default">
                <div className="relative">
                  <AgentIcon name={agent.name} emoji={agent.emoji} size={14} />
                  <div className={`absolute -bottom-0.5 -right-0.5 w-1.5 h-1.5 rounded-full border-white dark:border-gray-900 border ${
                    agentStatus[agent.name] ? "bg-green-500" : "bg-gray-300 dark:bg-gray-700"
                  }`} />
                </div>
                <span className={`uppercase tracking-tighter font-black ${
                  agentStatus[agent.name] ? "text-gray-700 dark:text-gray-300" : "text-gray-400"
                }`}>
                  {agent.name}
                </span>
              </div>
            ))}
          </div>
        )}

        {currentWorkDir && (
          <div className="flex items-center space-x-1.5 max-w-[200px] border-l border-gray-100 dark:border-gray-800 pl-4">
            <Terminal size={12} className="text-gray-400" />
            <span className="truncate opacity-60">{currentWorkDir}</span>
          </div>
        )}
      </div>

      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-1 px-2 py-0.5 bg-gray-50 dark:bg-gray-800 rounded-md border border-gray-100 dark:border-gray-700">
          <HardDrive size={10} className="text-gray-400" />
          <span className="font-black tracking-tight">HI CORE v0.1.0</span>
        </div>
      </div>
    </div>
  );
};
