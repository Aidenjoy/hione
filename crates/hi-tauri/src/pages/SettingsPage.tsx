import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getSettings, saveSettings } from '@/lib/tauri'
import { useAppStore } from '@/lib/store'
import i18n from '@/i18n'
import { useTranslation } from 'react-i18next'
import { 
  Languages, Moon, Sun, Monitor, 
  FolderSearch, Loader2, Save, CheckCircle
} from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'

export default function SettingsPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { theme, setTheme, language, setLanguage } = useAppStore()
  const [saveStatus, setSaveStatus] = useState(false)

  const [hiBinPath, setHiBinPath] = useState('')
  const [hiMonitorBinPath, setHiMonitorBinPath] = useState('')

  const { data: settings } = useQuery({
    queryKey: ['settings'],
    queryFn: getSettings
  })

  useEffect(() => {
    if (settings) {
      setHiBinPath(settings.hi_bin_path || '')
      setHiMonitorBinPath(settings.hi_monitor_bin_path || '')
    }
  }, [settings])

  const saveMutation = useMutation({
    mutationFn: () => saveSettings({
      language,
      theme,
      hi_bin_path: hiBinPath || undefined,
      hi_monitor_bin_path: hiMonitorBinPath || undefined
    }),
    onSuccess: () => {
      setSaveStatus(true)
      setTimeout(() => setSaveStatus(false), 3000)
      queryClient.invalidateQueries({ queryKey: ['settings'] })
    }
  })

  const handleLanguageChange = (lang: string) => {
    setLanguage(lang)
    i18n.changeLanguage(lang)
    saveMutation.mutate()
  }

  const handleThemeChange = (t: 'light' | 'dark' | 'system') => {
    setTheme(t)
    const root = document.documentElement
    if (t === 'dark') {
      root.classList.add('dark')
    } else if (t === 'light') {
      root.classList.remove('dark')
    } else {
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        root.classList.add('dark')
      } else {
        root.classList.remove('dark')
      }
    }
    saveMutation.mutate()
  }

  const handleBrowse = async (field: 'hi' | 'monitor') => {
    const selected = await open({ multiple: false, directory: false })
    if (selected && typeof selected === 'string') {
      if (field === 'hi') setHiBinPath(selected)
      else setHiMonitorBinPath(selected)
    }
  }

  return (
    <div className="w-full flex flex-col gap-6 animate-in fade-in duration-500">
      <header className="flex justify-between items-center">
        <h1 className="text-2xl font-black tracking-tight">
          {t('settings.title')}
        </h1>
        {saveStatus && (
          <div className="flex items-center space-x-2 text-green-500 font-bold animate-in fade-in slide-in-from-right-4">
            <CheckCircle size={18} />
            <span className="text-sm">{t('settings.saved')}</span>
          </div>
        )}
      </header>

      <div className="flex flex-col gap-8">
        {/* General Settings */}
        <section className="flex flex-col gap-4">
          <h2 className="text-xs font-black text-gray-400 uppercase tracking-widest flex items-center space-x-2 px-2">
            <Languages size={14} />
            <span>{t('settings.general')}</span>
          </h2>
          <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-8 flex flex-col gap-8 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <p className="font-black text-lg">{t('settings.language')}</p>
                <p className="text-sm text-gray-400 font-medium">{t('settings.languageDesc')}</p>
              </div>
              <select 
                value={language}
                onChange={(e) => handleLanguageChange(e.target.value)}
                className="bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-xl px-4 py-2.5 outline-none focus:ring-2 focus:ring-blue-500 font-bold text-sm cursor-pointer transition-all"
              >
                <option value="zh-CN">{t('settings.zhCN')}</option>
                <option value="en">{t('settings.en')}</option>
              </select>
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <p className="font-black text-lg">{t('settings.theme')}</p>
                <p className="text-sm text-gray-400 font-medium">{t('settings.themeDesc')}</p>
              </div>
              <div className="flex bg-gray-100 dark:bg-gray-800 p-1.5 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-inner">
                <button 
                  onClick={() => handleThemeChange('system')}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-xl transition-all ${theme === 'system' ? 'bg-white dark:bg-gray-700 shadow-md font-black' : 'text-gray-400 hover:text-gray-600'}`}
                >
                  <Monitor size={16} />
                  <span className="text-xs">{t('settings.system')}</span>
                </button>
                <button 
                  onClick={() => handleThemeChange('light')}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-xl transition-all ${theme === 'light' ? 'bg-white dark:bg-gray-700 shadow-md font-black text-blue-600' : 'text-gray-400 hover:text-gray-600'}`}
                >
                  <Sun size={16} />
                  <span className="text-xs">{t('settings.light')}</span>
                </button>
                <button 
                  onClick={() => handleThemeChange('dark')}
                  className={`flex items-center space-x-2 px-4 py-2 rounded-xl transition-all ${theme === 'dark' ? 'bg-white dark:bg-gray-700 shadow-md font-black text-blue-400' : 'text-gray-400 hover:text-gray-600'}`}
                >
                  <Moon size={16} />
                  <span className="text-xs">{t('settings.dark')}</span>
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* Path Config */}
        <section className="flex flex-col gap-4">
          <h2 className="text-xs font-black text-gray-400 uppercase tracking-widest flex items-center space-x-2 px-2">
            <FolderSearch size={14} />
            <span>{t('settings.pathConfig')}</span>
          </h2>
          <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-8 flex flex-col gap-8 shadow-sm">
            <div className="flex flex-col gap-6">
              <div className="flex flex-col gap-2">
                <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">{t('settings.hiBinPath')}</label>
                <div className="flex space-x-3">
                  <input 
                    type="text" 
                    value={hiBinPath}
                    onChange={(e) => setHiBinPath(e.target.value)}
                    className="flex-1 bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-5 py-3 outline-none focus:ring-2 focus:ring-blue-500 transition-all font-mono text-sm"
                    placeholder={t('settings.hiBinPathPlaceholder')}
                  />
                  <button 
                    onClick={() => handleBrowse('hi')}
                    className="px-6 py-3 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-2xl transition-colors font-black text-sm"
                  >
                    {t('settings.browse')}
                  </button>
                </div>
              </div>

              <div className="flex flex-col gap-2">
                <label className="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">{t('settings.hiMonitorBinPath')}</label>
                <div className="flex space-x-3">
                  <input 
                    type="text" 
                    value={hiMonitorBinPath}
                    onChange={(e) => setHiMonitorBinPath(e.target.value)}
                    className="flex-1 bg-gray-50 dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-2xl px-5 py-3 outline-none focus:ring-2 focus:ring-blue-500 transition-all font-mono text-sm"
                    placeholder={t('settings.hiMonitorBinPathPlaceholder')}
                  />
                  <button 
                    onClick={() => handleBrowse('monitor')}
                    className="px-6 py-3 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-2xl transition-colors font-black text-sm"
                  >
                    {t('settings.browse')}
                  </button>
                </div>
              </div>
            </div>

            <div className="pt-2">
              <button 
                onClick={() => saveMutation.mutate()}
                disabled={saveMutation.isPending}
                className="bg-blue-600 hover:bg-blue-700 text-white font-black px-10 py-3 rounded-2xl shadow-xl shadow-blue-200 dark:shadow-none flex items-center space-x-3 transition-all active:scale-95 disabled:opacity-50 text-sm"
              >
                {saveMutation.isPending ? <Loader2 className="animate-spin" size={20} /> : <Save size={20} />}
                <span>{t('settings.savePathSettings')}</span>
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>


  )
}
