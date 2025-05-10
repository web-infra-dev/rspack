import { useLang, withBase } from 'rspress/runtime';
import type DefaultKeys from './defaultKeys';

import { useCallback } from 'react';
import { EN_US } from './enUS';
import { PT_BR } from './ptBR';
import { ZH_CN } from './zhCN';

const translations = {
  en: EN_US,
  zh: ZH_CN,
  ptBR: PT_BR,
} as const;

export type LangTypes = keyof typeof translations;

export function useUrl(url: string) {
  const lang = useLang();
  return withBase(lang === 'zh' ? `/zh${url}` : url);
}
export function useI18nUrl() {
  const lang = useLang();

  const tUrl = useCallback(
    (url: string) => {
      return lang === 'en' ? url : `/${lang}${url}`;
    },
    [lang],
  );

  return tUrl;
}

export function useI18n() {
  const lang = useLang() as LangTypes;
  return (key: keyof DefaultKeys) => translations[lang][key];
}
