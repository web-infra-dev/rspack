import { Table } from '@builtIns/Table';
import React from 'react';
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
    en: 'Unsupported yet',
    zh: '暂未支持',
  },
  [SupportStatus.PartiallySupported]: {
    symbol: '🟡',
    en: 'Partially supported',
    zh: '部分支持',
  },
  [SupportStatus.FullySupported]: {
    symbol: '🟢',
    en: 'Supported',
    zh: '支持',
  },
};

interface PluginSupportStatus {
  name: string;
  status: SupportStatus;
  url?: string;
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
    url: '/plugins/webpack/banner-plugin',
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
    name: 'DefinePlugin',
    url: '/plugins/webpack/define-plugin',
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
    url: '/plugins/webpack/environment-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EvalSourceMapDevToolPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'HashedModuleIdsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'HotModuleReplacementPlugin',
    url: '/plugins/webpack/hot-module-replacement-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'IgnorePlugin',
    url: '/plugins/webpack/ignore-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'LimitChunkCountPlugin',
    url: '/plugins/webpack/limit-chunk-count-plugin',
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
    url: '/plugins/webpack/module-federation-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'NoEmitOnErrorsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'NormalModuleReplacementPlugin',
    url: '/plugins/webpack/normal-module-replacement-plugin',
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
    url: '/plugins/webpack/progress-plugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      zh: '仅支持 `profile` 选项',
      en: 'Only `profile` option supported',
    },
  },
  {
    name: 'ProvidePlugin',
    url: '/plugins/webpack/provide-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SourceMapDevToolPlugin',
    url: '/plugins/webpack/source-map-dev-tool-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SplitChunksPlugin',
    url: '/plugins/webpack/split-chunks-plugin',
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
    url: '/plugins/webpack/entry-plugin',
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
    url: '/plugins/webpack/node-template-plugin',
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
    url: '/plugins/webpack/node-target-plugin',
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
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`layer` and `wasmLoading` options are not supported, and `filename` cannot accept a function as a value',
      zh: '不支持 `layer`、`wasmLoading` 选项，`filename` 无法接受函数作为值',
    },
  },
  {
    name: 'ElectronTargetPlugin',
    url: '/plugins/webpack/electron-target-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableChunkLoadingPlugin',
    url: '/plugins/webpack/enable-chunk-loading-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableLibraryPlugin',
    url: '/plugins/webpack/enable-library-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableWasmLoadingPlugin',
    url: '/plugins/webpack/enable-wasm-loading-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EntryOptionPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ExternalsPlugin',
    url: '/plugins/webpack/externals-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'FetchCompileAsyncWasmPlugin',
    url: '/plugins/webpack/fetch-compile-async-wasm-plugin',
    status: SupportStatus.FullySupported,
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
    name: 'NaturalChunkIdsPlugin',
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
    status: SupportStatus.FullySupported,
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
    status: SupportStatus.FullySupported,
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
            width: '190px',
          },
        },
        {
          name: lang === 'zh' ? '备注' : 'Notes',
          key: 'notes',
        },
      ]}
      body={pluginSupportStatusList
        .sort((a, b) => {
          return (
            b.status - a.status ||
            (b.url && a.url ? 0 : (b.url?.length || 0) - (a.url?.length || 0))
          );
        })
        .map(({ name, url, status, notes }) => {
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
            name: url ? <a href={url}>{name}</a> : name,
            status: statusText,
            notes: notesText,
          };
        })}
    />
  );
};
