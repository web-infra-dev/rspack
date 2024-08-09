import { Table } from '@builtIns/Table';
import Markdown from 'markdown-to-jsx';
import type React from 'react';
import { useLang } from 'rspress/runtime';
import S from './PluginSupportStatusTable.module.scss';
import * as i18n from './i18n';

export enum CompatibleStatus {
  NotCompatible = 0,
  PartiallyCompatible = 1,
  Alternative = 2,
  Compatible = 3,
  Included = 4,
}

const SUPPORT_STATUS_LOCALIZED = {
  [CompatibleStatus.NotCompatible]: {
    symbol: '🔴',
    en: 'Incompatible',
    zh: '不兼容',
  },
  [CompatibleStatus.PartiallyCompatible]: {
    symbol: '🟡',
    en: 'Partially compatible',
    zh: '部分兼容',
  },
  [CompatibleStatus.Alternative]: {
    symbol: '🟡',
    en: 'Alternative',
    zh: '可替代',
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

const getNotesText = (
  lang: string,
  description: PluginSupportStatus['description'],
  status: PluginSupportStatus['status'],
) => {
  if (description) {
    return (
      <div>
        <Markdown>{description}</Markdown>
      </div>
    );
  }
  if (status === CompatibleStatus.NotCompatible) {
    return lang === 'zh' ? '待支持' : 'To be implemented';
  }
};

export const CommunityPluginCompatibleTable: React.FC = () => {
  const lang = useLang() as 'zh' | 'en';

  const pluginList: PluginSupportStatus[] = [
    {
      name: 'case-sensitive-paths-webpack-plugin',
      url: 'https://github.com/Urthen/case-sensitive-paths-webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['case-sensitive-paths-webpack-plugin-desc'],
    },
    {
      name: 'clean-webpack-plugin',
      url: 'https://github.com/johnagan/clean-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'moment-locales-webpack-plugin',
      url: 'https://www.npmjs.com/package/moment-locales-webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['moment-locales-webpack-plugin-desc'],
    },
    {
      name: 'copy-webpack-plugin',
      url: 'https://www.npmjs.com/package/copy-webpack-plugin',
      status: CompatibleStatus.Included,
      description: i18n[lang]['copy-plugin-desc'],
    },
    {
      name: 'compression-webpack-plugin',
      url: 'https://github.com/webpack-contrib/compression-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'css-minimizer-webpack-plugin',
      url: 'https://github.com/webpack-contrib/css-minimizer-webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['css-minimizer-webpack-plugin-desc'],
    },
    {
      name: 'eslint-webpack-plugin',
      url: 'https://github.com/webpack-contrib/eslint-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'fork-ts-checker-webpack-plugin',
      url: 'https://github.com/TypeStrong/fork-ts-checker-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'html-webpack-plugin',
      url: 'https://www.npmjs.com/package/html-webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['html-webpack-plugin-desc'],
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
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['terser-webpack-plugin-desc'],
    },
    {
      name: 'html-minimizer-webpack-plugin',
      url: 'https://github.com/webpack-contrib/html-minimizer-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'json-minimizer-webpack-plugin',
      url: 'https://github.com/webpack-contrib/json-minimizer-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'stylelint-webpack-plugin',
      url: 'https://github.com/webpack-contrib/stylelint-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'tsconfig-paths-webpack-plugin',
      url: 'https://www.npmjs.com/package/tsconfig-paths-webpack-plugin',
      status: CompatibleStatus.Included,
      description: i18n[lang]['tsconfig-paths-webpack-plugin-desc'],
    },
    {
      name: 'webpack-bundle-analyzer',
      url: 'https://www.npmjs.com/package/webpack-bundle-analyzer',
      status: CompatibleStatus.Compatible,
    },
    {
      name: '@vanilla-extract/webpack-plugin',
      url: 'https://github.com/vanilla-extract-css/vanilla-extract',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'webpack-stats-plugin',
      url: 'https://www.npmjs.com/package/webpack-stats-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'license-webpack-plugin',
      url: 'https://www.npmjs.com/package/license-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'monaco-editor-webpack-plugin',
      url: 'https://www.npmjs.com/package/monaco-editor-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'dotenv-webpack',
      url: 'https://www.npmjs.com/package/dotenv-webpack',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'friendly-errors-webpack-plugin',
      url: 'https://www.npmjs.com/package/friendly-errors-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: '@nx/webpack',
      url: 'https://www.npmjs.com/package/@nx/webpack',
      status: CompatibleStatus.Alternative,
      description: i18n[lang]['@nx/webpack-desc'],
    },
    {
      name: 'webpack-filter-warnings-plugin',
      url: 'https://github.com/mattlewis92/webpack-filter-warnings-plugin',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['webpack-filter-warnings-plugin-desc'],
    },
    {
      name: 'speed-measure-webpack-plugin',
      url: 'https://www.npmjs.com/package/speed-measure-webpack-plugin',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['speed-measure-webpack-plugin-desc'],
    },
    {
      name: 'circular-dependency-plugin',
      url: 'https://github.com/aackerman/circular-dependency-plugin',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['circular-dependency-plugin-desc'],
    },
    {
      name: 'critters-webpack-plugin',
      url: 'https://github.com/GoogleChromeLabs/critters',
      status: CompatibleStatus.NotCompatible,
    },
    {
      name: 'html-webpack-tags-plugin',
      url: 'https://github.com/jharris4/html-webpack-tags-plugin',
      status: CompatibleStatus.PartiallyCompatible,
      description: i18n[lang]['html-webpack-tags-plugin-desc'],
    },
    {
      name: '@loadable/webpack-plugin',
      url: 'https://www.npmjs.com/package/@loadable/webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'error-overlay-webpack-plugin',
      url: 'https://github.com/gregberge/error-overlay-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'webpackbar',
      url: 'https://www.npmjs.com/package/webpackbar',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['progress-plugin-function-un-support-desc'],
    },
    {
      name: 'progress-bar-webpack-plugin',
      url: 'https://www.npmjs.com/package/progress-bar-webpack-plugin',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['progress-plugin-function-un-support-desc'],
    },
    {
      name: 'image-minimizer-webpack-plugin',
      url: 'https://www.npmjs.com/package/image-minimizer-webpack-plugin',
      status: CompatibleStatus.PartiallyCompatible,
      description: i18n[lang]['image-minimizer-webpack-plugin-desc'],
    },
    {
      name: 'webpack-manifest-plugin',
      url: 'https://github.com/shellscape/webpack-manifest-plugin',
      status: CompatibleStatus.Alternative,
      description: i18n[lang]['webpack-manifest-plugin-desc'],
    },
    {
      name: 'webpack-subresource-integrity',
      url: 'https://github.com/waysact/webpack-subresource-integrity',
      status: CompatibleStatus.NotCompatible,
    },
    {
      name: 'pnp-webpack-plugin',
      url: 'https://github.com/arcanis/pnp-webpack-plugin',
      status: CompatibleStatus.NotCompatible,
      description: i18n[lang]['resolve-plugin-un-support-desc'],
    },
    {
      name: 'webpack-virtual-modules',
      url: 'https://github.com/sysgears/webpack-virtual-modules',
      status: CompatibleStatus.Alternative,
      description: i18n[lang]['webpack-virtual-modules-desc'],
    },
    {
      name: 'node-polyfill-webpack-plugin',
      url: 'https://www.npmjs.com/package/node-polyfill-webpack-plugin',
      status: CompatibleStatus.Compatible,
    },
    {
      name: 'workbox-webpack-plugin',
      url: 'https://www.npmjs.com/package/workbox-webpack-plugin',
      status: CompatibleStatus.Alternative,
      description: i18n[lang]['workbox-webpack-plugin-desc'],
    },
    {
      name: '@pmmmwh/react-refresh-webpack-plugin',
      url: 'https://www.npmjs.com/package/@pmmmwh/react-refresh-webpack-plugin',
      status: CompatibleStatus.Alternative,
      description: i18n[lang]['react-refresh-webpack-plugin-desc'],
    },
    {
      name: '@sentry/webpack-plugin',
      url: 'https://www.npmjs.com/package/@sentry/webpack-plugin',
      status: CompatibleStatus.Compatible,
      description: i18n[lang]['sentry_webpack-plugin-desc'],
    },
    {
      name: 'serwist',
      url: 'https://github.com/serwist/serwist',
      status: CompatibleStatus.Compatible,
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
            width: '150px',
          },
        },
        {
          name: lang === 'zh' ? '备注' : 'Notes',
          key: 'notes',
        },
      ]}
      body={pluginList
        .sort((a, b) => b.status - a.status || a.name.localeCompare(b.name))
        .map(({ name, url, status, description }) => {
          const { symbol, en, zh } = SUPPORT_STATUS_LOCALIZED[status];
          const statusText = `${symbol} ${lang === 'zh' ? zh : en}`;

          return {
            name: (
              <a href={url} target="_blank" rel="noreferrer">
                {name}
              </a>
            ),
            status: statusText,
            notes: getNotesText(lang, description, status),
          };
        })}
    />
  );
};
