import { create } from 'zustand'
import type { SessionInfo } from '../types'

interface AppStore {
  theme: 'light' | 'dark' | 'system'
  language: string
  currentWorkDir: string | null
  sessionConnected: boolean
  sessionInfo: SessionInfo | null
  setTheme: (t: 'light' | 'dark' | 'system') => void
  setLanguage: (l: string) => void
  setWorkDir: (d: string | null) => void
  setSessionConnected: (v: boolean) => void
  setSessionInfo: (s: SessionInfo | null) => void
}

export const useAppStore = create<AppStore>((set) => ({
  theme: 'system',
  language: 'zh-CN',
  currentWorkDir: null,
  sessionConnected: false,
  sessionInfo: null,
  setTheme: (theme) => set({ theme }),
  setLanguage: (language) => set({ language }),
  setWorkDir: (currentWorkDir) => set({ currentWorkDir }),
  setSessionConnected: (sessionConnected) => set({ sessionConnected }),
  setSessionInfo: (sessionInfo) => set({ sessionInfo }),
}))
