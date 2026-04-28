/**
 * Sidebar Menu Visibility Configuration
 * Set to true to show, false to hide.
 */
export const MENU_CONFIG = {
  launcher: true,
  tasks: true,
  tools: true,
  agents: false,      // Currently hidden
  mcp: false,         // Currently hidden
  skills: false,      // Currently hidden
  customTools: false, // Currently hidden
  help: true,
  about: true,
  settings: true,
} as const;

export type MenuKey = keyof typeof MENU_CONFIG;
