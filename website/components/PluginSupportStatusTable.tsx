import { Table } from '@builtIns/Table';
import { useLang } from '@rspress/core/runtime';
import { Link } from '@rspress/core/theme';
import { useI18nUrl } from '@theme/i18n';
import type React from 'react';

enum SupportStatus {
  NotSupported = 0,
  PartiallySupported = 1,
  FullySupported = 2,
}

const SUPPORT_STATUS_LOCALIZED = {
  [SupportStatus.NotSupported]: {
    symbol: '🔴',
    en: 'Not supported',
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
    name: 'AsyncWebAssemblyModulesPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: 'Used internally, but not exposed through the JavaScript API',
      zh: '已在内部使用，但未通过 JavaScript API 导出',
    },
  },
  {
    name: 'BannerPlugin',
    url: '/plugins/banner-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ChunkPrefetchPreloadPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: 'Used internally, but not exposed through the JavaScript API',
      zh: '已在内部使用，但未通过 JavaScript API 导出',
    },
  },
  {
    name: 'CssModulesPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: 'Used internally, but not exposed through the JavaScript API',
      zh: '已在内部使用，但未通过 JavaScript API 导出',
    },
  },
  {
    name: 'DefinePlugin',
    url: '/plugins/define-plugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: '`rspack.DefinePlugin.runtimeValue` is not supported',
      zh: '不支持 `rspack.DefinePlugin.runtimeValue` 函数',
    },
  },
  {
    name: 'DllPlugin',
    url: '/plugins/dll-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'DotenvPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'EnvironmentPlugin',
    url: '/plugins/environment-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EvalSourceMapDevToolPlugin',
    url: '/plugins/eval-source-map-dev-tool-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'HashedModuleIdsPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'HotModuleReplacementPlugin',
    url: '/plugins/hot-module-replacement-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'HtmlModulesPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'IgnorePlugin',
    url: '/plugins/ignore-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'LimitChunkCountPlugin',
    url: '/plugins/limit-chunk-count-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ManifestPlugin',
    status: SupportStatus.NotSupported,
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
    url: '/plugins/module-federation-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'NoEmitOnErrorsPlugin',
    url: '/plugins/no-emit-on-errors-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'NormalModuleReplacementPlugin',
    url: '/plugins/normal-module-replacement-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'PlatformPlugin',
    status: SupportStatus.NotSupported,
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
    url: '/plugins/progress-plugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      zh: '仅支持部分选项',
      en: 'Only some options are supported',
    },
  },
  {
    name: 'ProvidePlugin',
    url: '/plugins/provide-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SingleEntryPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: 'Use `EntryPlugin` instead',
      zh: '请改用 `EntryPlugin`',
    },
  },
  {
    name: 'SourceMapDevToolPlugin',
    url: '/plugins/source-map-dev-tool-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SplitChunksPlugin',
    url: '/plugins/split-chunks-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'VirtualUrlPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'WatchIgnorePlugin',
    status: SupportStatus.NotSupported,
  },

  // internal webpack plugins
  {
    name: 'NodeEnvironmentPlugin',
    url: '/plugins/low-level-plugins#nodeenvironmentplugin',
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
    url: '/plugins/entry-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'JsonpTemplatePlugin',
    url: '/plugins/low-level-plugins#jsonp-template-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'NodeTemplatePlugin',
    url: '/plugins/low-level-plugins#node-template-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'LibraryTemplatePlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'WebWorkerTemplatePlugin',
    url: '/plugins/low-level-plugins#web-worker-template-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EvalDevToolModulePlugin',
    url: '/plugins/low-level-plugins#evaldevtoolmoduleplugin',
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
    url: '/plugins/low-level-plugins#node-target-plugin',
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

  // other webpack plugins
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
    url: '/plugins/low-level-plugins#consumesharedplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ContainerPlugin',
    url: '/plugins/low-level-plugins#containerplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ContainerReferencePlugin',
    url: '/plugins/low-level-plugins#containerreferenceplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ContextExclusionPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ContextReplacementPlugin',
    url: '/plugins/context-replacement-plugin',
    status: SupportStatus.FullySupported,
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
    url: '/plugins/deterministic-module-ids-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'DllReferencePlugin',
    url: '/plugins/dll-reference-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'DynamicEntryPlugin',
    url: '/plugins/low-level-plugins#dynamicentryplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ElectronTargetPlugin',
    url: '/plugins/low-level-plugins#electron-target-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableChunkLoadingPlugin',
    url: '/plugins/low-level-plugins#enable-chunk-loading-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableLibraryPlugin',
    url: '/plugins/low-level-plugins#enable-library-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EnableWasmLoadingPlugin',
    url: '/plugins/low-level-plugins#enable-wasm-loading-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'EntryOptionPlugin',
    url: '/plugins/low-level-plugins#entryoptionplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ExternalsPlugin',
    url: '/plugins/externals-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'FetchCompileAsyncWasmPlugin',
    url: '/plugins/low-level-plugins#fetchcompileasyncwasmplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'FetchCompileWasmPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'HttpUriPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'JavascriptModulesPlugin',
    url: '/plugins/javascript-modules-plugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      zh: '静态方法 `getCompilationHooks()` 的返回值未支持所有 hook',
      en: 'Static `getCompilationHooks()` does not expose all hooks',
    },
  },
  {
    name: 'LibManifestPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'LoaderOptionsPlugin',
    url: '/plugins/low-level-plugins#loaderoptionsplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'LoaderTargetPlugin',
    url: '/plugins/low-level-plugins#loadertargetplugin',
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
      en: '`context` is not supported',
      zh: '不支持 `context` 选项',
    },
  },
  {
    name: 'NaturalModuleIdsPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'OccurrenceChunkIdsPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'OccurrenceModuleIdsPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'ProvideSharedPlugin',
    url: '/plugins/low-level-plugins#providesharedplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SharePlugin',
    url: '/plugins/low-level-plugins#shareplugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'ReadFileCompileAsyncWasmPlugin',
    status: SupportStatus.PartiallySupported,
    notes: {
      en: 'Used internally, but not exposed through the JavaScript API',
      zh: '已在内部使用，但未通过 JavaScript API 导出',
    },
  },
  {
    name: 'ReadFileCompileWasmPlugin',
    status: SupportStatus.NotSupported,
  },
  {
    name: 'RuntimeChunkPlugin',
    url: '/plugins/runtime-chunk-plugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SideEffectsFlagPlugin',
    status: SupportStatus.FullySupported,
  },
  {
    name: 'SyncModuleIdsPlugin',
    status: SupportStatus.FullySupported,
  },
].sort((a, b) => {
  return b.status - a.status || a.name.localeCompare(b.name);
});

const getNotesText = (
  lang: string,
  notes: PluginSupportStatus['notes'],
  status: PluginSupportStatus['status'],
) => {
  if (notes) {
    return lang === 'zh' ? notes.zh : notes.en;
  }
  if (status === SupportStatus.NotSupported) {
    return lang === 'zh' ? '待实现' : 'Not implemented';
  }
};

export const PluginSupportStatusTable: React.FC = () => {
  const lang = useLang();
  const tUrl = useI18nUrl();

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
            width: '190px',
          },
        },
        {
          name: lang === 'zh' ? '备注' : 'Notes',
          key: 'notes',
        },
      ]}
      body={pluginSupportStatusList.map(({ name, url, status, notes }) => {
        const { symbol, en, zh } = SUPPORT_STATUS_LOCALIZED[status];
        const statusText = `${symbol} ${lang === 'zh' ? zh : en}`;

        return {
          name: url ? <Link href={tUrl(url)}>{name}</Link> : name,
          status: statusText,
          notes: getNotesText(lang, notes, status),
        };
      })}
    />
  );
};
