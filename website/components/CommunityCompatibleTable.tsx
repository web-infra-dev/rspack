import React from 'react';
import { Table } from '@builtIns/Table';
import { useLang } from 'rspress/runtime';
import S from './PluginSupportStatusTable.module.scss';
import Markdown from 'markdown-to-jsx';
import * as i18n from './i18n';

export enum CompatibleStatus {
  NotCompatible,
  Compatible,
  Included,
}

const SUPPORT_STATUS_LOCALIZED = {
  [CompatibleStatus.NotCompatible]: {
    symbol: '🔴',
    en: 'Not Compatible',
    zh: '不兼容',
  },
  [CompatibleStatus.Compatible]: {
    symbol: '🟢',
    en: 'Compatible',
    zh: '兼容',
  },
  [CompatibleStatus.Included]: {
    symbol: '🔵',
    en: 'Included',
    zh: '已内置',
  },
};

export interface PluginSupportStatus {
  name: string;
  status: CompatibleStatus;
  url: string;
  description?: string;
}

export const CommunityPluginCompatibleTable: React.FC = () => {
  const lang = useLang() as 'zh' | 'en';

  const pluginList: PluginSupportStatus[] = [
    {
      name: 'html-webpack-plugin',
      url: 'https://www.npmjs.com/package/html-webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['html-webpack-plugin-desc'],
    },
    {
      name: '@sentry/webpack-plugin',
      url: 'https://www.npmjs.com/package/@sentry/webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['sentry_webpack-plugin-desc'],
    },
    {
      name: 'case-sensitive-paths-webpack-plugin',
      url: 'https://github.com/Urthen/case-sensitive-paths-webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['case-sensitive-paths-webpack-plugin-desc'],
    },
    {
      name: 'css-minimizer-webpack-plugin',
      url: 'https://github.com/webpack-contrib/css-minimizer-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'eslint-webpack-plugin',
      url: 'https://github.com/webpack-contrib/eslint-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'webpack-manifest-plugin',
      url: 'https://github.com/shellscape/webpack-manifest-plugin',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['webpack-manifest-plugin-desc'],
    },
    {
      name: 'copy-webpack-plugin',
      url: 'https://www.npmjs.com/package/copy-webpack-plugin',
      status: CompatibleStatus.Included,
      description: i18n[lang]['copy-plugin-desc'],
    },
    {
      name: 'mini-css-extract-plugin',
      url: 'https://webpack.js.org/plugins/mini-css-extract-plugin',
      status: CompatibleStatus.Included,
      description: i18n[lang]['mini-css-extract-plugin-desc'],
    },
    {
      name: 'terser-webpack-plugin',
      url: 'https://webpack.js.org/plugins/terser-webpack-plugin',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['terser-webpack-plugin-desc'],
    },
    {
      name: 'webpack-bundle-analyzer',
      url: 'https://www.npmjs.com/package/webpack-bundle-analyzer',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'tsconfig-paths-webpack-plugin',
      url: 'https://www.npmjs.com/package/tsconfig-paths-webpack-plugin',
      status: CompatibleStatus.Included,
      description: i18n[lang]['tsconfig-paths-webpack-plugin-desc'],
    },
    {
      name: 'fork-ts-checker-webpack-plugin',
      url: 'https://github.com/TypeStrong/fork-ts-checker-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'webpack-subresource-integrity',
      url: 'https://github.com/waysact/webpack-subresource-integrity',
      status: CompatibleStatus.NotCompatible,
    },
  ];

  return (
    <Table
      className={S.PluginSupportStatusTable}
      header={[
        {
          name: lang === 'zh' ? '插件' : 'Plugin',
          key: 'name',
        },
        {
          name: lang === 'zh' ? '支持情况' : 'Support status',
          key: 'status',
          style: {
            width: '200px',
          },
        },
        {
          name: lang === 'zh' ? '备注' : 'Notes',
          key: 'notes',
        },
      ]}
      body={pluginList
        .sort((a, b) => b.status - a.status)
        .map(({ name, url, status, description }) => {
          const { symbol, en, zh } = SUPPORT_STATUS_LOCALIZED[status];
          const statusText = `${symbol} ${lang === 'zh' ? zh : en}`;

          const notesText = (() => {
            if (description) {
              return (
                <div className={S.PluginNote}>
                  <Markdown>{description}</Markdown>
                </div>
              );
            }
            if (status === CompatibleStatus.NotCompatible) {
              return lang === 'zh' ? '待支持' : 'To be implemented';
            }
          })();

          return {
            name: (
              <a className={S.PluginLink} href={url}>
                {name}
              </a>
            ),
            status: statusText,
            notes: notesText,
          };
        })}
    />
  );
};
