import { Github, Shield, Heart, Code2, Zap, Terminal } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export default function AboutPage() {
  const { t } = useTranslation();
  
  return (
    <div className="h-full flex flex-col animate-in fade-in zoom-in-95 duration-500 overflow-y-auto scrollbar-hide">
      {/* Hero Section */}
      <div className="flex flex-col lg:flex-row items-center justify-between gap-10 px-8 py-12">
        
        {/* Left Side: Branding */}
        <div className="flex flex-col items-center lg:items-start text-center lg:text-left space-y-5 max-w-md">
          <div className="relative group">
            <div className="absolute -inset-1 bg-gradient-to-r from-blue-600 to-cyan-500 rounded-3xl blur opacity-25 group-hover:opacity-50 transition duration-1000 group-hover:duration-200"></div>
            <div className="relative w-28 h-28 bg-white dark:bg-gray-900 rounded-3xl flex items-center justify-center shadow-xl border border-gray-100 dark:border-gray-800 overflow-hidden">
              <img src="/logo.png" alt="hione logo" className="w-full h-full object-cover" />
            </div>
          </div>
          
          <div className="space-y-1.5">
            <h1 className="text-5xl font-black tracking-tighter text-gray-900 dark:text-white italic">
              {t('about.title')}
            </h1>
            <p className="text-xs font-black text-blue-500 uppercase tracking-[0.3em] pl-1">
              {t('about.subtitle')}
            </p>
          </div>

          <p className="text-base text-gray-500 dark:text-gray-400 font-medium leading-relaxed">
            {t('about.description')}
          </p>

          <p className="text-sm text-blue-600 dark:text-blue-400 font-semibold bg-blue-50 dark:bg-blue-900/20 px-4 py-2 rounded-xl inline-block border border-blue-100 dark:border-blue-800/50">
            {t('about.starMe')}
          </p>

          <div className="flex items-center gap-4 pt-3">
            <a 
              href="https://github.com/Aidenjoy/hione" 
              target="_blank" 
              rel="noopener noreferrer"
              className="p-3 bg-gray-50 dark:bg-gray-800 hover:bg-blue-50 dark:hover:bg-blue-900/30 rounded-2xl border border-gray-100 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:text-blue-600 transition-all"
            >
              <Github size={20} />
            </a>
          </div>
        </div>

        {/* Right Side: Features Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 w-full max-w-xl">
          <FeatureCard 
            icon={<Shield className="text-blue-500" size={24} />}
            title={t('about.features.privacy.title')}
            desc={t('about.features.privacy.desc')}
          />
          <FeatureCard 
            icon={<Zap className="text-yellow-500" size={24} />}
            title={t('about.features.speed.title')}
            desc={t('about.features.speed.desc')}
          />
          <FeatureCard 
            icon={<Terminal className="text-green-500" size={24} />}
            title={t('about.features.multimodal.title')}
            desc={t('about.features.multimodal.desc')}
          />
          <FeatureCard 
            icon={<Code2 className="text-purple-500" size={24} />}
            title={t('about.features.opensource.title')}
            desc={t('about.features.opensource.desc')}
          />
        </div>
      </div>

      {/* Bottom Bar */}
      <div className="p-8 border-t border-gray-100 dark:border-gray-800 flex flex-col sm:flex-row items-center justify-between gap-4 mt-auto">
        <div className="flex items-center gap-6">
          <div className="flex flex-col">
            <span className="text-[10px] font-black text-gray-400 uppercase tracking-widest">{t('about.version')}</span>
            <span className="text-sm font-bold text-gray-900 dark:text-white">{t('about.versionValue')}</span>
          </div>
          <div className="w-px h-8 bg-gray-100 dark:bg-gray-800 hidden sm:block"></div>
          <div className="flex flex-col">
            <span className="text-[10px] font-black text-gray-400 uppercase tracking-widest">{t('about.platform')}</span>
            <span className="text-sm font-bold text-gray-900 dark:text-white">{t('about.platformValue')}</span>
          </div>
        </div>

        <div className="text-right">
          <p className="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider flex items-center justify-end gap-1.5">
            <Heart size={12} className="text-red-500 fill-red-500" /> {t('about.crafted')}
          </p>
          <p className="text-[10px] text-gray-500 dark:text-gray-500 mt-1.5 font-medium tracking-tight">{t('about.est')}</p>
        </div>
      </div>
    </div>
  );
}

function FeatureCard({ icon, title, desc }: { icon: React.ReactNode, title: string, desc: string }) {
  return (
    <div className="p-6 bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-3xl shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all duration-300 flex flex-col gap-4 group">
      <div className="w-12 h-12 bg-gray-50 dark:bg-gray-950 rounded-2xl flex items-center justify-center transition-transform group-hover:scale-110">
        {icon}
      </div>
      <div>
        <h3 className="text-base font-black text-gray-900 dark:text-white mb-1 tracking-tight">{title}</h3>
        <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed font-medium">{desc}</p>
      </div>
    </div>
  );
}

