import type { Compiler } from '../Compiler';
import type { ExternalsType } from '../config';
import type { ShareFallback } from '../sharing/IndependentSharedPlugin';
import type { SharedConfig } from '../sharing/SharePlugin';
import { TreeShakingSharedPlugin } from '../sharing/TreeShakingSharedPlugin';
import { isRequiredVersion } from '../sharing/utils';
import {
  ModuleFederationManifestPlugin,
  type ModuleFederationManifestPluginOptions,
} from './ModuleFederationManifestPlugin';
import type { ModuleFederationPluginV1Options } from './ModuleFederationPluginV1';
import { ModuleFederationRuntimePlugin } from './ModuleFederationRuntimePlugin';
import { parseOptions } from './options';

declare const MF_RUNTIME_CODE: string;

export interface ModuleFederationPluginOptions extends Omit<
  ModuleFederationPluginV1Options,
  'enhanced'
> {
  runtimePlugins?: RuntimePlugins;
  implementation?: string;
  shareStrategy?: 'version-first' | 'loaded-first';
  manifest?: ModuleFederationManifestPluginOptions;
  injectTreeShakingUsedExports?: boolean;
  treeShakingSharedDir?: string;
  treeShakingSharedExcludePlugins?: string[];
  treeShakingSharedPlugins?: string[];
}
export type RuntimePlugins = string[] | [string, Record<string, unknown>][];

export class ModuleFederationPlugin {
  private _treeShakingSharedPlugin?: TreeShakingSharedPlugin;

  constructor(private _options: ModuleFederationPluginOptions) {}

  apply(compiler: Compiler) {
    const { webpack } = compiler;
    const paths = getPaths(this._options);
    compiler.options.resolve.alias = {
      '@module-federation/runtime-tools': paths.runtimeTools,
      '@module-federation/runtime': paths.runtime,
      ...compiler.options.resolve.alias,
    };

    const sharedOptions = getSharedOptions(this._options);
    const treeShakingEntries = sharedOptions.filter(
      ([, config]) => config.treeShaking,
    );
    if (treeShakingEntries.length > 0) {
      this._treeShakingSharedPlugin = new TreeShakingSharedPlugin({
        mfConfig: this._options,
        secondary: false,
      });
      this._treeShakingSharedPlugin.apply(compiler);
    }

    // need to wait treeShakingSharedPlugin buildAssets
    let runtimePluginApplied = false;
    compiler.hooks.beforeRun.tap(
      {
        name: 'ModuleFederationPlugin',
        stage: 100,
      },
      () => {
        if (runtimePluginApplied) return;
        runtimePluginApplied = true;
        const entryRuntime = getDefaultEntryRuntime(
          paths,
          this._options,
          compiler,
          this._treeShakingSharedPlugin?.buildAssets,
        );
        new ModuleFederationRuntimePlugin({
          entryRuntime,
        }).apply(compiler);
      },
    );
    compiler.hooks.watchRun.tap(
      {
        name: 'ModuleFederationPlugin',
        stage: 100,
      },
      () => {
        if (runtimePluginApplied) return;
        runtimePluginApplied = true;
        const entryRuntime = getDefaultEntryRuntime(
          paths,
          this._options,
          compiler,
          this._treeShakingSharedPlugin?.buildAssets || {},
        );
        new ModuleFederationRuntimePlugin({
          entryRuntime,
        }).apply(compiler);
      },
    );

    new webpack.container.ModuleFederationPluginV1({
      ...this._options,
      enhanced: true,
    }).apply(compiler);

    if (this._options.manifest) {
      new ModuleFederationManifestPlugin(this._options).apply(compiler);
    }
  }
}

interface RuntimePaths {
  runtimeTools: string;
  bundlerRuntime: string;
  runtime: string;
}

interface RemoteInfo {
  alias: string;
  name?: string;
  entry?: string;
  externalType: ExternalsType;
  shareScope: string;
}

type RemoteInfos = Record<string, RemoteInfo[]>;

export function getRemoteInfos(
  options: ModuleFederationPluginOptions,
): RemoteInfos {
  if (!options.remotes) {
    return {};
  }

  function extractUrlAndGlobal(urlAndGlobal: string) {
    const index = urlAndGlobal.indexOf('@');
    if (index <= 0 || index === urlAndGlobal.length - 1) {
      return null;
    }
    return [
      urlAndGlobal.substring(index + 1),
      urlAndGlobal.substring(0, index),
    ] as const;
  }

  function getExternalTypeFromExternal(external: string) {
    if (/^[a-z0-9-]+ /.test(external)) {
      const idx = external.indexOf(' ');
      return [
        external.slice(0, idx) as ExternalsType,
        external.slice(idx + 1),
      ] as const;
    }
    return null;
  }

  function getExternal(external: string) {
    const result = getExternalTypeFromExternal(external);
    if (result === null) {
      return [remoteType, external] as const;
    }
    return result;
  }

  const remoteType =
    options.remoteType ||
    (options.library ? (options.library.type as ExternalsType) : 'script');

  const remotes = parseOptions(
    options.remotes,
    (item) => ({
      external: Array.isArray(item) ? item : [item],
      shareScope: options.shareScope || 'default',
    }),
    (item) => ({
      external: Array.isArray(item.external) ? item.external : [item.external],
      shareScope: item.shareScope || options.shareScope || 'default',
    }),
  );

  const remoteInfos: Record<string, RemoteInfo[]> = {};
  for (const [key, config] of remotes) {
    for (const external of config.external) {
      const [externalType, externalRequest] = getExternal(external);
      remoteInfos[key] ??= [];
      if (externalType === 'script') {
        const [url, global] = extractUrlAndGlobal(externalRequest)!;
        remoteInfos[key].push({
          alias: key,
          name: global,
          entry: url,
          externalType,
          shareScope: config.shareScope,
        });
      } else {
        remoteInfos[key].push({
          alias: key,
          name: undefined,
          entry: undefined,
          externalType,
          shareScope: config.shareScope,
        });
      }
    }
  }
  return remoteInfos;
}

function getRuntimePlugins(options: ModuleFederationPluginOptions) {
  return options.runtimePlugins ?? [];
}

function getSharedOptions(
  options: ModuleFederationPluginOptions,
): [string, SharedConfig][] {
  if (!options.shared) return [];
  return parseOptions<SharedConfig, SharedConfig>(
    options.shared,
    (item, key) => {
      if (typeof item !== 'string') {
        throw new Error('Unexpected array in shared');
      }
      return item === key || !isRequiredVersion(item)
        ? { import: item }
        : { import: key, requiredVersion: item };
    },
    (item) => item,
  );
}

function getPaths(options: ModuleFederationPluginOptions): RuntimePaths {
  if (IS_BROWSER) {
    return {
      runtimeTools: '@module-federation/runtime-tools',
      bundlerRuntime: '@module-federation/webpack-bundler-runtime',
      runtime: '@module-federation/runtime',
    };
  }

  const runtimeToolsPath =
    options.implementation ??
    require.resolve('@module-federation/runtime-tools');
  const bundlerRuntimePath = require.resolve(
    '@module-federation/webpack-bundler-runtime',
    { paths: [runtimeToolsPath] },
  );
  const runtimePath = require.resolve('@module-federation/runtime', {
    paths: [runtimeToolsPath],
  });
  return {
    runtimeTools: runtimeToolsPath,
    bundlerRuntime: bundlerRuntimePath,
    runtime: runtimePath,
  };
}

function getDefaultEntryRuntime(
  paths: RuntimePaths,
  options: ModuleFederationPluginOptions,
  compiler: Compiler,
  treeShakingShareFallbacks?: ShareFallback,
) {
  const runtimePlugins = getRuntimePlugins(options);
  const remoteInfos = getRemoteInfos(options);
  const runtimePluginImports = [];
  const runtimePluginVars = [];
  const libraryType = options.library?.type || 'var';
  for (let i = 0; i < runtimePlugins.length; i++) {
    const runtimePluginVar = `__module_federation_runtime_plugin_${i}__`;
    const pluginSpec = runtimePlugins[i];
    const pluginPath = Array.isArray(pluginSpec) ? pluginSpec[0] : pluginSpec;
    const pluginParams = Array.isArray(pluginSpec) ? pluginSpec[1] : undefined;

    runtimePluginImports.push(
      `import ${runtimePluginVar} from ${JSON.stringify(pluginPath)}`,
    );
    const paramsCode =
      pluginParams === undefined ? 'undefined' : JSON.stringify(pluginParams);
    runtimePluginVars.push(`${runtimePluginVar}(${paramsCode})`);
  }

  const content = [
    `import __module_federation_bundler_runtime__ from ${JSON.stringify(
      paths.bundlerRuntime,
    )}`,
    ...runtimePluginImports,
    `const __module_federation_runtime_plugins__ = [${runtimePluginVars.join(
      ', ',
    )}]`,
    `const __module_federation_remote_infos__ = ${JSON.stringify(remoteInfos)}`,
    `const __module_federation_container_name__ = ${JSON.stringify(
      options.name ?? compiler.options.output.uniqueName,
    )}`,
    `const __module_federation_share_strategy__ = ${JSON.stringify(
      options.shareStrategy ?? 'version-first',
    )}`,
    `const __module_federation_share_fallbacks__ = ${JSON.stringify(
      treeShakingShareFallbacks,
    )}`,
    `const __module_federation_library_type__ = ${JSON.stringify(libraryType)}`,
    IS_BROWSER
      ? MF_RUNTIME_CODE
      : compiler.webpack.Template.getFunctionContent(
          require('./moduleFederationDefaultRuntime.js'),
        ),
  ].join(';');
  return `@module-federation/runtime/rspack.js!=!data:text/javascript,${content}`;
}
