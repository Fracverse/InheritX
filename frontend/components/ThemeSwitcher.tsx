'use client';

import { useTheme } from '@/context/ThemeContext';
import { Sun, Moon, Monitor } from 'lucide-react';

export function ThemeSwitcher() {
  const { theme, setTheme } = useTheme();

  return (
    <div className="flex items-center gap-1 rounded-lg bg-white/5 border border-white/10 p-1">
      <button
        onClick={() => setTheme('light')}
        title="Light theme"
        className={`p-2 rounded-md transition-colors ${
          theme === 'light'
            ? 'bg-violet-500 text-white'
            : 'text-slate-400 hover:text-white hover:bg-white/5'
        }`}
      >
        <Sun size={18} />
      </button>
      <button
        onClick={() => setTheme('dark')}
        title="Dark theme"
        className={`p-2 rounded-md transition-colors ${
          theme === 'dark'
            ? 'bg-violet-500 text-white'
            : 'text-slate-400 hover:text-white hover:bg-white/5'
        }`}
      >
        <Moon size={18} />
      </button>
      <button
        onClick={() => setTheme('system')}
        title="System theme"
        className={`p-2 rounded-md transition-colors ${
          theme === 'system'
            ? 'bg-violet-500 text-white'
            : 'text-slate-400 hover:text-white hover:bg-white/5'
        }`}
      >
        <Monitor size={18} />
      </button>
    </div>
  );
}
