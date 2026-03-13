import { createRequire } from 'node:module';
import { dirname, isAbsolute } from 'node:path';
import type { Compiler } from '../Compiler';
import type { ExternalsType } from '../config';
import type { ShareFallback } from '../sharing/IndependentSharedPlugin';
import type { SharedConfig, ShareScope } from '../sharing/SharePlugin';
import { TreeShakingSharedPlugin } from '../sharing/TreeShakingSharedPlugin';
import { isRequiredVersion } from '../sharing/utils';
import {
  ModuleFederationManifestPlugin,
  type ModuleFederationManifestPluginOptions,
} from './ModuleFederationManifestPlugin';
import type { ModuleFederationPluginV1Options } from './ModuleFederationPluginV1';
import {
  type ModuleFederationRuntimeExperimentsOptions,
  ModuleFederationRuntimePlugin,
} from './ModuleFederationRuntimePlugin';
import { parseOptions } from './options';

const require = createRequire(import.meta.url);

declare const MF_RUNTIME_CODE: string;
const RSC_BRIDGE_EXPOSE = './__rspack_rsc_bridge__';
const RSC_LAYER = 'react-server-components';
const SSR_LAYER = 'server-side-rendering';
const RSC_SSR_EXPOSE_PREFIX = './__rspack_rsc_ssr__/';
const RSC_BRIDGE_RUNTIME_PLUGIN_SPECIFIER =
  '@module-federation/webpack-bundler-runtime/rsc-bridge-runtime-plugin';
const RSC_BRIDGE_EXPOSE_SPECIFIER =
  '@module-federation/webpack-bundler-runtime/rsc-bridge-expose';

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
  experiments?: ModuleFederationRuntimeExperimentsOptions;
}
export type RuntimePlugins = string[] | [string, Record<string, unknown>][];

export class ModuleFederationPlugin {
  private _treeShakingSharedPlugin?: TreeShakingSharedPlugin;

  constructor(private _options: ModuleFederationPluginOptions) {}

  apply(compiler: Compiler) {
    const target = compiler.options.target;
    const isRscMfOptIn = this._options.experiments?.rsc === true;
    const enableRscBridge = isRscMfOptIn && target !== false;
    const paths = getPaths(this._options, compiler, enableRscBridge);
    const options =
      enableRscBridge && paths.rscBridgeExpose
        ? augmentRscBridgeExposes(this._options, paths.rscBridgeExpose)
        : this._options;
    if (enableRscBridge && !paths.rscBridgeExpose) {
      throw new Error(
        '[ModuleFederationPlugin] Unable to resolve Module Federation RSC bridge expose module. Ensure @module-federation/webpack-bundler-runtime is up to date.',
      );
    }

    const { webpack } = compiler;
    compiler.options.resolve.alias = {
      '@module-federation/runtime-tools': paths.runtimeTools,
      '@module-federation/runtime': paths.runtime,
      ...compiler.options.resolve.alias,
    };

    const sharedOptions = getSharedOptions(options);
    const treeShakingEntries = sharedOptions.filter(
      ([, config]) => config.treeShaking,
    );
    if (treeShakingEntries.length > 0) {
      this._treeShakingSharedPlugin = new TreeShakingSharedPlugin({
        mfConfig: options,
        secondary: false,
      });
      this._treeShakingSharedPlugin.apply(compiler);
    }

    const asyncStartup = options.experiments?.asyncStartup ?? false;
    const runtimeExperiments: ModuleFederationRuntimeExperimentsOptions = {
      asyncStartup,
      rsc: enableRscBridge,
    };

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
          options,
          compiler,
          this._treeShakingSharedPlugin?.buildAssets,
          enableRscBridge,
        );
        new ModuleFederationRuntimePlugin({
          entryRuntime,
          experiments: runtimeExperiments,
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
          options,
          compiler,
          this._treeShakingSharedPlugin?.buildAssets || {},
          enableRscBridge,
        );
        // Pass only the entry runtime to the Rust-side plugin
        new ModuleFederationRuntimePlugin({
          entryRuntime,
          experiments: runtimeExperiments,
        }).apply(compiler);
      },
    );

    // Keep v1 options isolated from v2-only fields like `experiments`.
    const v1Options: ModuleFederationPluginV1Options = {
      name: options.name,
      exposes: options.exposes,
      filename: options.filename,
      library: options.library,
      remoteType: options.remoteType,
      remotes: options.remotes,
      runtime: options.runtime,
      shareScope: options.shareScope,
      shared: options.shared,
      enhanced: true,
    };
    new webpack.container.ModuleFederationPluginV1(v1Options).apply(compiler);

    if (options.manifest) {
      new ModuleFederationManifestPlugin(options).apply(compiler);
    }
  }
}

interface RuntimePaths {
  runtimeTools: string;
  bundlerRuntime: string;
  runtime: string;
  rscBridgeRuntimePlugin?: string;
  rscBridgeExpose?: string;
}

interface RemoteInfo {
  alias: string;
  name?: string;
  entry?: string;
  externalType: ExternalsType;
  shareScope: ShareScope;
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
      shareScope: options.shareScope ?? 'default',
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

function getPaths(
  options: ModuleFederationPluginOptions,
  compiler: Compiler,
  resolveRscBridge = false,
): RuntimePaths {
  if (IS_BROWSER) {
    return {
      runtimeTools: '@module-federation/runtime-tools',
      bundlerRuntime: '@module-federation/webpack-bundler-runtime',
      runtime: '@module-federation/runtime',
      rscBridgeRuntimePlugin: resolveRscBridge
        ? RSC_BRIDGE_RUNTIME_PLUGIN_SPECIFIER
        : undefined,
      rscBridgeExpose: resolveRscBridge
        ? RSC_BRIDGE_EXPOSE_SPECIFIER
        : undefined,
    };
  }

  let runtimeToolsPath: string;
  if (options.implementation) {
    runtimeToolsPath = options.implementation;
  } else {
    try {
      runtimeToolsPath = require.resolve('@module-federation/runtime-tools', {
        paths: [compiler.context],
      });
    } catch (e) {
      if ((e as NodeJS.ErrnoException).code === 'MODULE_NOT_FOUND') {
        throw new Error(
          'Module Federation runtime is not installed. Please install it by running:\n\n  npm install @module-federation/runtime-tools\n',
        );
      }
      throw e;
    }
  }
  const runtimeResolvePaths = Array.from(
    new Set([
      compiler.context,
      runtimeToolsPath,
      ...(isAbsolute(runtimeToolsPath) ? [dirname(runtimeToolsPath)] : []),
    ]),
  );
  const bundlerRuntimePath = require.resolve(
    '@module-federation/webpack-bundler-runtime',
    { paths: runtimeResolvePaths },
  );
  const runtimePath = require.resolve('@module-federation/runtime', {
    paths: runtimeResolvePaths,
  });

  let rscBridgeRuntimePluginPath: string | undefined;
  let rscBridgeExposePath: string | undefined;
  if (resolveRscBridge) {
    rscBridgeRuntimePluginPath = require.resolve(
      RSC_BRIDGE_RUNTIME_PLUGIN_SPECIFIER,
      { paths: runtimeResolvePaths },
    );
    rscBridgeExposePath = require.resolve(RSC_BRIDGE_EXPOSE_SPECIFIER, {
      paths: runtimeResolvePaths,
    });
  }

  return {
    runtimeTools: runtimeToolsPath,
    bundlerRuntime: bundlerRuntimePath,
    runtime: runtimePath,
    rscBridgeRuntimePlugin: rscBridgeRuntimePluginPath,
    rscBridgeExpose: rscBridgeExposePath,
  };
}

function getDefaultEntryRuntime(
  paths: RuntimePaths,
  options: ModuleFederationPluginOptions,
  compiler: Compiler,
  treeShakingShareFallbacks?: ShareFallback,
  enableRscBridge?: boolean,
) {
  const configuredRuntimePlugins = getRuntimePlugins(options);
  const rscBridgeRuntimePluginPath = paths.rscBridgeRuntimePlugin;
  const isRscBridgeRuntimePlugin = (pluginPath: unknown): boolean => {
    if (typeof pluginPath !== 'string') {
      return false;
    }
    if (
      pluginPath === rscBridgeRuntimePluginPath ||
      pluginPath === RSC_BRIDGE_RUNTIME_PLUGIN_SPECIFIER
    ) {
      return true;
    }
    const isBridgeRuntimeFilename =
      pluginPath.endsWith('/rsc-bridge-runtime-plugin.cjs') ||
      pluginPath.endsWith('\\rsc-bridge-runtime-plugin.cjs');
    if (isBridgeRuntimeFilename) {
      return true;
    }
    return false;
  };
  const runtimePlugins =
    enableRscBridge && rscBridgeRuntimePluginPath
      ? configuredRuntimePlugins.some((pluginSpec) => {
          const pluginPath = Array.isArray(pluginSpec)
            ? pluginSpec[0]
            : pluginSpec;
          return isRscBridgeRuntimePlugin(pluginPath);
        })
        ? configuredRuntimePlugins
        : configuredRuntimePlugins.concat(rscBridgeRuntimePluginPath)
      : configuredRuntimePlugins;
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
    runtimePluginVars.push(
      `{ plugin: ${runtimePluginVar}, params: ${paramsCode} }`,
    );
  }

  const content = [
    `import __module_federation_bundler_runtime__ from ${JSON.stringify(
      paths.bundlerRuntime,
    )}`,
    ...runtimePluginImports,
    `const __module_federation_container_name__ = ${JSON.stringify(
      options.name ?? compiler.options.output.uniqueName,
    )}`,
    `const __module_federation_runtime_plugins__ = [${runtimePluginVars.join(
      ', ',
    )}].filter(({ plugin }) => plugin).map(({ plugin, params }) => plugin(params))`,
    `const __module_federation_remote_infos__ = ${JSON.stringify(remoteInfos)}`,
    `const __module_federation_share_strategy__ = ${JSON.stringify(
      options.shareStrategy ?? 'version-first',
    )}`,
    `const __module_federation_share_fallbacks__ = ${JSON.stringify(
      treeShakingShareFallbacks,
    )}`,
    `const __module_federation_library_type__ = ${JSON.stringify(libraryType)}`,
    IS_BROWSER
      ? MF_RUNTIME_CODE
      : compiler.rspack.Template.getFunctionContent(
          require('./moduleFederationDefaultRuntime.js').default,
        ),
  ].join(';');
  return `@module-federation/runtime/rspack.js!=!data:text/javascript,${encodeURIComponent(content)}`;
}

function augmentRscBridgeExposes(
  options: ModuleFederationPluginOptions,
  bridgeModulePath: string,
): ModuleFederationPluginOptions {
  if (!options.exposes) {
    return {
      ...options,
      exposes: {
        [RSC_BRIDGE_EXPOSE]: {
          import: bridgeModulePath,
          layer: RSC_LAYER,
        },
      },
    };
  }

  const exposesEntries = parseOptions(
    options.exposes,
    (item) => ({
      import: Array.isArray(item) ? item : [item],
      name: undefined,
      layer: undefined,
    }),
    (item) => ({
      import: Array.isArray(item.import) ? item.import : [item.import],
      name: item.name || undefined,
      layer: item.layer || undefined,
    }),
  );
  if (exposesEntries.some(([key]) => key === RSC_BRIDGE_EXPOSE)) {
    return options;
  }

  const exposeObject = Object.fromEntries(
    exposesEntries.map(([key, value]) => [
      key,
      value.name
        ? value.layer
          ? { import: value.import, name: value.name, layer: value.layer }
          : { import: value.import, name: value.name }
        : value.layer
          ? {
              import:
                value.import.length === 1 ? value.import[0] : value.import,
              layer: value.layer,
            }
          : value.import.length === 1
            ? value.import[0]
            : value.import,
    ]),
  );

  for (const [key, value] of exposesEntries) {
    if (key === RSC_BRIDGE_EXPOSE || key.startsWith(RSC_SSR_EXPOSE_PREFIX)) {
      continue;
    }
    const normalizedKey = key.startsWith('./') ? key.slice(2) : key;
    const hiddenSsrExposeKey = `${RSC_SSR_EXPOSE_PREFIX}${normalizedKey}`;
    if (
      Object.prototype.hasOwnProperty.call(exposeObject, hiddenSsrExposeKey)
    ) {
      continue;
    }
    exposeObject[hiddenSsrExposeKey] = value.name
      ? {
          import: value.import,
          name: value.name,
          layer: SSR_LAYER,
        }
      : {
          import: value.import.length === 1 ? value.import[0] : value.import,
          layer: SSR_LAYER,
        };
  }

  return {
    ...options,
    exposes: {
      ...exposeObject,
      [RSC_BRIDGE_EXPOSE]: {
        import: bridgeModulePath,
        layer: RSC_LAYER,
      },
    },
  };
}
