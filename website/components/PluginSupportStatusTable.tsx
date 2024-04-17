import React from 'react';
import { Table } from '@builtIns/Table';
import { useLang } from 'rspress/runtime';

enum SupportStatus {
  NotSupported,
  PartiallySupported,
  FullySupported,
}

const SUPPORT_STATUS_LOCALIZED = {
  [SupportStatus.NotSupported]: {
    symbol: '🔴',
    en: 'Not Supported',
    zh: '不支持',
  },
  [SupportStatus.PartiallySupported]: {
    symbol: '🟡',
    en: 'Partially Supported',
    zh: '部分支持',
  },
  [SupportStatus.FullySupported]: {
    symbol: '🟢',
    en: 'Fully Supported',
    zh: '完全支持',
  },
};

interface PluginSupportStatus {
  name: string;
  status: SupportStatus;
  notes?: {
    en: string;
    zh: string;
  };
}

const pluginSupportStatusList: PluginSupportStatus[] = [
  {
    name: 'AutomaticPrefetchPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'BannerPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`stage` option not supported',
      zh: '不支持 `stage` 选项',
    },
  },
  {
    name: 'ContextExclusionPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ContextReplacementPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'DefinePlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`rspack.DefinePlugin.runtimeValue` function not supported',
      zh: '不支持 `rspack.DefinePlugin.runtimeValue` 函数',
    },
  },
  {
    name: 'DllPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'EnvironmentPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EvalSourceMapDevToolPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`test`, `include`, `exclude`, `moduleFilenameTemplate`, `protocol` options not supported',
      zh: '不支持 `test`、`include`、`exclude`、`moduleFilenameTemplate`、`protocol` 选项',
    },
  },
  {
    name: 'HashedModuleIdsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'HotModuleReplacementPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'IgnorePlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'LimitChunkCountPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'MinChunkSizePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ModuleConcatenationPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ModuleFederationPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'NoEmitOnErrorsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'NormalModuleReplacementPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'PrefetchPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ProfilingPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ProgressPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      zh: '仅支持 `profile` 选项',
      en: 'Only `profile` option supported',
    },
  },
  {
    name: 'ProvidePlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SourceMapDevToolPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SplitChunksPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`minSizeReduction`, `usedExports` options not supported',
      zh: '不支持 `minSizeReduction`、`usedExports` 选项',
    },
  },
  {
    name: 'WatchIgnorePlugin',
    status: SupportStatus.NotSupported,
  },
];

export const PluginSupportStatusTable: React.FC = () => {
  const lang = useLang();

  return (
    <Table
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
      body={pluginSupportStatusList
        .sort((a, b) => b.status - a.status)
        .map(({ name, status, notes }) => {
          const { symbol, en, zh } = SUPPORT_STATUS_LOCALIZED[status];
          const statusText = `${symbol} ${lang === 'zh' ? zh : en}`;

          const notesText = (() => {
            if (notes) {
              return lang === 'zh' ? notes.zh : notes.en;
            }
            if (status === SupportStatus.NotSupported) {
              return lang === 'zh' ? '待实现' : 'To be implemented';
            }
          })();

          return {
            name,
            status: statusText,
            notes: notesText,
          };
        })}
    />
  );
};
