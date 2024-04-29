import React from 'react';
import { Table } from '@builtIns/Table';
import { useLang } from 'rspress/runtime';
import S from './PluginSupportStatusTable.module.scss';

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

  // internal Rspack plugins
  {
    name: 'NodeEnvironmentPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'MemoryCachePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'RecordIdsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'EntryPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`layer` and `wasmLoading` options are not supported, and `filename` cannot accept a function as a value',
      zh: '不支持 `layer`、`wasmLoading` 选项，`filename` 无法接受函数作为值',
    },
  },
  {
    name: 'JsonpTemplatePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'NodeTemplatePlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'LibraryTemplatePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'WebWorkerTemplatePlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EvalDevToolModulePlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'APIPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ConstPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'RequireJsStuffPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'NodeSourcePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'NodeTargetPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'AMDPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'CommonJsPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'RequireContextPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'RequireEnsurePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'RequireIncludePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'MergeDuplicateChunksPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'RemoveEmptyChunksPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'FlagIncludedChunksPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'RealContentHashPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`hashFunction` and `hashDigest` options are not supported',
      zh: '不支持 `hashFunction`、`hashDigest` 选项',
    },
  },

  // not write in webpack docs
  {
    name: 'AbstractLibraryPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'AggressiveMergingPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'AggressiveSplittingPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ChunkModuleIdRangePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'CleanPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ConsumeSharedPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ContainerPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ContainerReferencePlugin',
    status: SupportStatus.FullySupported,
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
    name: 'DelegatedPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'DeterministicChunkIdsPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`context` and `maxLength` options are not supported',
      zh: '不支持 `context`、`maxLength` 选项',
    },
  },
  {
    name: 'DeterministicModuleIdsPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`context`, `test`, `maxLength`, `salt`, `fixedLength`, `failOnConflict` options are not supported',
      zh: '不支持 `context`、`test`、`maxLength`、`salt`、`fixedLength`、`failOnConflict` 选项',
    },
  },
  {
    name: 'DllReferencePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'DynamicEntryPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ElectronTargetPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableChunkLoadingPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableLibraryPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableWasmLoadingPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EntryOptionPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`layer`, `chunkLoading`, `wasmLoading`, `library` options are not supported, and `entry` and `filename` cannot accept a function as a value',
      zh: '不支持 `layer`、`chunkLoading`、`wasmLoading`、`library` 选项，`entry` 和 `filename` 无法接受函数作为值',
    },
  },
  {
    name: 'ExternalsPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'FetchCompileAsyncWasmPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: 'Temporarily not exported from the JavaScript side',
      zh: '暂时未从 JavaScript 侧导出',
    },
  },
  {
    name: 'FetchCompileWasmPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'HttpUriPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'JavascriptModulesPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'LibManifestPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'LoaderOptionsPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'NamedChunkIdsPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`delimiter` and `context` options are not supported',
      zh: '不支持 `delimiter`、`context` 选项',
    },
  },
  {
    name: 'NamedModuleIdsPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`context` options are not supported',
      zh: '不支持 `context` 选项',
    },
  },
  {
    name: 'NaturalModuleIdsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'OccurrenceChunkIdsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'OccurrenceModuleIdsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ProvideSharedPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: 'Temporarily not exported from the JavaScript side',
      zh: '暂时未从 JavaScript 侧导出',
    },
  },
  {
    name: 'ReadFileCompileWasmPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'RuntimeChunkPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'SideEffectsFlagPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SyncModuleIdsPlugin',
    status: SupportStatus.NotSupported,
  },
];

export const PluginSupportStatusTable: React.FC = () => {
  const lang = useLang();

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
