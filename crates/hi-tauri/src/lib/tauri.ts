import { invoke } from '@tauri-apps/api/core'
import { 
  SetupStatus, ToolInfo, Agent, McpServer, Skill, SkillRepo, 
  RecentSession, SessionInfo, TaskRecord, CustomTool, AppSettings 
} from '../types'

// setup
export const checkSetup = () => invoke<SetupStatus>('check_setup')
export const installDependency = (name: string) => invoke<void>('install_dependency', { name })
export const installBundledCli = () => invoke<string>('install_bundled_cli')

// tools
export const listTools = () => invoke<ToolInfo[]>('list_tools')
export const installTool = (name: string) => invoke<void>('install_tool', { name })
export const uninstallTool = (name: string) => invoke<void>('uninstall_tool', { name })
export const checkToolUpdate = (name: string) => invoke<string | null>('check_tool_update', { name })

// agent
export const listAgents = () => invoke<Agent[]>('list_agents')
export const saveAgent = (agent: Agent) => invoke<void>('save_agent', { agent })
export const testAgentConnection = (name: string) => invoke<boolean>('test_agent_connection', { name })

// mcp
export const listMcpServers = () => invoke<McpServer[]>('list_mcp_servers')
export const createMcpServer = (server: McpServer) => invoke<void>('create_mcp_server', { server })
export const updateMcpServer = (server: McpServer) => invoke<void>('update_mcp_server', { server })
export const deleteMcpServer = (id: string) => invoke<void>('delete_mcp_server', { id })
export const toggleMcpForAgent = (serverId: string, agentName: string, enabled: boolean) => invoke<void>('toggle_mcp_for_agent', { serverId, agentName, enabled })
export const syncMcpToTools = () => invoke<void>('sync_mcp_to_tools')

// skill
export const listSkills = () => invoke<Skill[]>('list_skills')
export const listSkillRepos = () => invoke<SkillRepo[]>('list_skill_repos')
export const addSkillRepo = (url: string) => invoke<SkillRepo>('add_skill_repo', { url })
export const removeSkillRepo = (id: string) => invoke<void>('remove_skill_repo', { id })
export const installSkill = (repoId: string, skillName: string) => invoke<void>('install_skill', { repoId, skillName })
export const deleteSkill = (id: string) => invoke<void>('delete_skill', { id })
export const toggleSkillForAgent = (skillId: string, agentName: string, enabled: boolean) => invoke<void>('toggle_skill_for_agent', { skillId, agentName, enabled })
export const syncSkillsToTools = () => invoke<void>('sync_skills_to_tools')

// session
export const listRecentSessions = () => invoke<RecentSession[]>('list_recent_sessions')
export const launchSession = (workDir: string, tools: string[], auto: boolean, resume: boolean) => invoke<void>('launch_session', { workDir, tools, auto, resume })
export const connectSession = (workDir: string) => invoke<SessionInfo>('connect_session', { workDir })
export const disconnectSession = () => invoke<void>('disconnect_session')
export const killSession = (workDir: string) => invoke<void>('kill_session', { workDir })
export const detectSession = (workDir: string) => invoke<SessionInfo | null>('detect_session', { workDir })

// task
export const listTasks = () => invoke<TaskRecord[]>('list_tasks')
export const pushTask = (target: string, content: string) => invoke<string>('push_task', { target, content })
export const cancelTask = (taskId: string) => invoke<void>('cancel_task', { taskId })
export const checkAgent = (name: string) => invoke<boolean>('check_agent', { name })

// custom_tools
export const readCustomTools = (workDir: string) => invoke<CustomTool[]>('read_custom_tools', { workDir })
export const writeCustomTools = (workDir: string, tools: CustomTool[]) => invoke<void>('write_custom_tools', { workDir, tools })

// settings
export const getSettings = () => invoke<AppSettings>('get_settings')
export const saveSettings = (settings: AppSettings) => invoke<void>('save_settings', { settings })
