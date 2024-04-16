import { withBase, useLang } from 'rspress/runtime';
import { EN_US } from './enUS';
import { ZH_CN } from './zhCN';

const translations = {
  en: EN_US,
  zh: ZH_CN,
} as const;

export function useUrl(url: string) {
  const lang = useLang();
  return withBase(lang === 'zh' ? url : `/en${url}`);
}

export function useI18n() {
  const lang = useLang() as keyof typeof translations;
  return (key: keyof typeof EN_US) => translations[lang][key];
}
