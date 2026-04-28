export interface SetupStatus { tmux: boolean; node: boolean; rust: boolean; hi: boolean; hi_monitor: boolean }
export interface ToolInfo { name: string; installed: boolean; version?: string }
export interface Agent { id: string; name: string; api_key?: string; api_base_url?: string; model?: string; extra_config: Record<string, unknown>; enabled: boolean }
export interface McpServer { id: string; name: string; server_config: Record<string, unknown>; enabled_for: string[] }
export interface Skill { id: string; name: string; repo_url?: string; local_path?: string; enabled_for: string[]; installed_at: number }
export interface SkillRepo { id: string; url: string; name: string }
export interface RecentSession { work_dir: string; tools: string[]; auto_mode: boolean; resume_mode: boolean; last_used: number }
export interface TaskRecord { id: string; sender: string; receiver: string; content: string; status: string; created_at: number }
export interface CustomTool { name: string; auto_flags: string[]; resume_flags: string[] }
export interface AppSettings { language: string; theme: 'light' | 'dark' | 'system'; hi_bin_path?: string; hi_monitor_bin_path?: string }

export interface WindowInfo {
  index: number;
  name: string;
  command: string;
  launch_command: string;
  auto_mode: boolean;
  resume_mode: boolean;
  is_main: boolean;
  pid?: number;
  tmux_pane_id?: string;
}

export interface SessionInfo {
  id: string;
  windows: WindowInfo[];
  work_dir: string;
  hione_dir: string;
  socket_path: string;
  monitor_pid?: number;
}
