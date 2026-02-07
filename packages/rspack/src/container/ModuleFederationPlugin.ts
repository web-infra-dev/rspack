import { createRequire } from 'node:module';
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
import {
  type ModuleFederationRuntimeExperimentsOptions,
  ModuleFederationRuntimePlugin,
} from './ModuleFederationRuntimePlugin';
import { parseOptions } from './options';

const require = createRequire(import.meta.url);

declare const MF_RUNTIME_CODE: string;
const RSC_BRIDGE_EXPOSE = './__rspack_rsc_bridge__';
const RSC_LAYER = 'react-server-components';
const NODE_RUNTIME_PLUGIN_REQUEST = '@module-federation/node/runtimePlugin';

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
    const isNodeLikeBuild = isNodeLikeTarget(target);
    const isRscMfOptIn = this._options.experiments?.rsc === true;
    if (isRscMfOptIn && isNodeLikeBuild) {
      validateRscMfOptions(this._options, compiler);
    }
    const isRscMfEnabled = isRscMfOptIn && isNodeLikeBuild;
    const options = isRscMfEnabled
      ? augmentRscBridgeExposes(this._options)
      : this._options;

    const { webpack } = compiler;
    const paths = getPaths(options, compiler);
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
      rsc: isRscMfEnabled,
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
          isRscMfEnabled,
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
          isRscMfEnabled,
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
}

interface RemoteInfo {
  alias: string;
  name?: string;
  entry?: string;
  externalType: ExternalsType;
  shareScope: string | string[];
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

function getPaths(
  options: ModuleFederationPluginOptions,
  compiler: Compiler,
): RuntimePaths {
  if (IS_BROWSER) {
    return {
      runtimeTools: '@module-federation/runtime-tools',
      bundlerRuntime: '@module-federation/webpack-bundler-runtime',
      runtime: '@module-federation/runtime',
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
  enableRscBridge?: boolean,
) {
  const runtimePlugins = getRuntimePlugins(options);
  const remoteInfos = getRemoteInfos(options);
  const runtimePluginImports = [];
  const runtimePluginVars = [];
  if (enableRscBridge) {
    runtimePluginVars.push(
      `{ plugin: (${getInlineRscRuntimePluginFactorySource()})(${JSON.stringify(
        getRscActionProxyModuleId(options, compiler),
      )}), params: undefined }`,
    );
  }
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
      : compiler.webpack.Template.getFunctionContent(
          require('./moduleFederationDefaultRuntime.js').default,
        ),
  ].join(';');
  return `@module-federation/runtime/rspack.js!=!data:text/javascript,${encodeURIComponent(content)}`;
}

function getTargetValues(target: unknown): string[] {
  if (Array.isArray(target)) {
    return target.filter((item): item is string => typeof item === 'string');
  }
  return typeof target === 'string' ? [target] : [];
}

function isNodeLikeTarget(target: unknown): boolean {
  if (target === false) return true;
  const targets = getTargetValues(target);
  return targets.some((value) => value.includes('node'));
}

function hasAsyncNodeTarget(target: unknown): boolean {
  const targets = getTargetValues(target);
  return targets.some(
    (value) => value === 'async-node' || value.startsWith('async-node'),
  );
}

function hasNodeRuntimePlugin(
  runtimePlugins: RuntimePlugins | undefined,
): boolean {
  if (!runtimePlugins || runtimePlugins.length === 0) {
    return false;
  }
  const normalize = (value: string) => value.replace(/\\/g, '/').toLowerCase();
  return runtimePlugins.some((pluginSpec) => {
    const pluginPath = Array.isArray(pluginSpec) ? pluginSpec[0] : pluginSpec;
    if (typeof pluginPath !== 'string') return false;
    const normalized = normalize(pluginPath);
    return (
      normalized.includes(normalize(NODE_RUNTIME_PLUGIN_REQUEST)) ||
      (normalized.includes('@module-federation/node') &&
        normalized.includes('runtimeplugin'))
    );
  });
}

function validateRscMfOptions(
  options: ModuleFederationPluginOptions,
  compiler: Compiler,
) {
  const errors: string[] = [];
  if (!hasAsyncNodeTarget(compiler.options.target)) {
    errors.push('`target` must include `"async-node"`.');
  }
  if (options.experiments?.asyncStartup !== true) {
    errors.push('`experiments.asyncStartup` must be `true`.');
  }
  if (!hasNodeRuntimePlugin(options.runtimePlugins)) {
    errors.push(
      '`runtimePlugins` must include `@module-federation/node/runtimePlugin`.',
    );
  }

  if (errors.length === 0) return;

  throw new Error(
    [
      '[ModuleFederationPlugin] Invalid RSC federation server configuration.',
      ...errors.map((item, index) => `${index + 1}. ${item}`),
      '',
      'Expected configuration snippet:',
      'new rspack.container.ModuleFederationPlugin({',
      '  // ...',
      '  runtimePlugins: [require.resolve("@module-federation/node/runtimePlugin")],',
      '  experiments: {',
      '    asyncStartup: true,',
      '    rsc: true,',
      '  },',
      '});',
      '',
      'And compiler target should be "async-node".',
    ].join('\n'),
  );
}

function augmentRscBridgeExposes(
  options: ModuleFederationPluginOptions,
): ModuleFederationPluginOptions {
  const bridgeModulePath = getRscBridgeExposeRequest();
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

  return {
    ...options,
    exposes: {
      ...Object.fromEntries(
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
      ),
      [RSC_BRIDGE_EXPOSE]: {
        import: bridgeModulePath,
        layer: RSC_LAYER,
      },
    },
  };
}

function getRscActionProxyModuleId(
  options: ModuleFederationPluginOptions,
  compiler: Compiler,
): string {
  const containerName = options.name ?? compiler.options.output.uniqueName;
  return `__rspack_mf_rsc_action_proxy__:${containerName || 'container'}`;
}

function getRscBridgeExposeRequest(): string {
  const bridgeExposeSource = [
    'const actionReferenceCache = Object.create(null)',
    'let scannedExposesForActions = false',
    'function isObject(value) { return typeof value === "object" && value !== null; }',
    'function cacheActionReferencesFromExports(exports) {',
    '  if (typeof exports === "function" && typeof exports.$$id === "string") {',
    '    actionReferenceCache[exports.$$id] = exports;',
    '  }',
    '  if (!isObject(exports)) return;',
    '  for (const value of Object.values(exports)) {',
    '    if (typeof value === "function" && typeof value.$$id === "string") {',
    '      actionReferenceCache[value.$$id] = value;',
    '    }',
    '  }',
    '}',
    'async function scanExposedModulesForActions() {',
    '  if (scannedExposesForActions) return;',
    '  scannedExposesForActions = true;',
    '  const moduleMap = __webpack_require__.initializeExposesData && __webpack_require__.initializeExposesData.moduleMap;',
    '  if (!isObject(moduleMap)) return;',
    '  for (const [exposeName, getFactory] of Object.entries(moduleMap)) {',
    '    if (exposeName === "./__rspack_rsc_bridge__" || typeof getFactory !== "function") continue;',
    '    try {',
    '      const factory = await getFactory();',
    '      const exports = typeof factory === "function" ? factory() : factory;',
    '      cacheActionReferencesFromExports(exports);',
    '      if (isObject(exports) && isObject(exports.default)) {',
    '        cacheActionReferencesFromExports(exports.default);',
    '      }',
    '    } catch (_error) {}',
    '  }',
    '}',
    'export function getManifest() { return __webpack_require__.rscM; }',
    'export async function executeAction(actionId, args) {',
    '  await scanExposedModulesForActions();',
    '  const action = actionReferenceCache[actionId];',
    '  if (typeof action !== "function") {',
    '    throw new Error("[ModuleFederationPlugin.rsc] Missing remote action for id \\"" + actionId + "\\". Ensure it is reachable from a federated expose.");',
    '  }',
    '  return action(...(Array.isArray(args) ? args : []));',
    '}',
  ].join(';');
  return `@module-federation/runtime/rspack.js!=!data:text/javascript,${encodeURIComponent(bridgeExposeSource)}`;
}

function getInlineRscRuntimePluginFactorySource(): string {
  return [
    'function createRspackRscRuntimePluginFactory(proxyModuleId) {',
    "  var RSC_BRIDGE_EXPOSE = '__rspack_rsc_bridge__';",
    "  var ACTION_PREFIX = 'remote:';",
    "  var MODULE_PREFIX = 'remote-module:';",
    '  var bridgePromises = Object.create(null);',
    '  var actionMap = Object.create(null);',
    '  var mergedRemoteAliases = new Set();',
    '  var proxyModuleInstalled = false;',
    '  var isObject = function(value) { return typeof value === "object" && value !== null; };',
    '  var stableStringify = function(value) {',
    '    try {',
    '      return JSON.stringify(value, function(_key, current) {',
    '        if (Array.isArray(current)) return current;',
    '        if (!isObject(current)) return current;',
    '        return Object.fromEntries(Object.keys(current).sort().map(function(key) { return [key, current[key]]; }));',
    '      });',
    '    } catch (_error) {',
    '      return String(value);',
    '    }',
    '  };',
    '  var ensureHostManifest = function() {',
    '    if (!isObject(__webpack_require__.rscM)) {',
    '      __webpack_require__.rscM = {};',
    '    }',
    '    var manifest = __webpack_require__.rscM;',
    '    manifest.serverManifest = isObject(manifest.serverManifest) ? manifest.serverManifest : {};',
    '    manifest.clientManifest = isObject(manifest.clientManifest) ? manifest.clientManifest : {};',
    '    manifest.serverConsumerModuleMap = isObject(manifest.serverConsumerModuleMap) ? manifest.serverConsumerModuleMap : {};',
    '    return manifest;',
    '  };',
    '  var ensureBridge = async function(alias) {',
    '    if (!alias) {',
    "      throw new Error('[ModuleFederationPlugin.rsc] Failed to resolve remote alias for RSC bridge.');",
    '    }',
    '    if (bridgePromises[alias]) {',
    '      return bridgePromises[alias];',
    '    }',
    '    var instance = __webpack_require__.federation && __webpack_require__.federation.instance;',
    '    if (!instance || typeof instance.loadRemote !== "function") {',
    "      throw new Error('[ModuleFederationPlugin.rsc] Module Federation runtime instance is unavailable while loading the RSC bridge.');",
    '    }',
    "    bridgePromises[alias] = Promise.resolve(instance.loadRemote(alias + '/' + RSC_BRIDGE_EXPOSE)).then(function(bridge) {",
    '      if (!bridge || typeof bridge.getManifest !== "function" || typeof bridge.executeAction !== "function") {',
    "        throw new Error('[ModuleFederationPlugin.rsc] Remote \"' + alias + '\" is missing the internal RSC bridge expose.');",
    '      }',
    '      return bridge;',
    '    });',
    '    return bridgePromises[alias];',
    '  };',
    '  var ensureActionProxyModule = function() {',
    '    if (proxyModuleInstalled) return;',
    '    proxyModuleInstalled = true;',
    '    __webpack_require__.m[proxyModuleId] = function(module) {',
    '      module.exports = new Proxy({}, {',
    '        get: function(_target, property) {',
    '          if (typeof property !== "string") return undefined;',
    '          if (!Object.prototype.hasOwnProperty.call(actionMap, property)) return undefined;',
    '          var mapping = actionMap[property];',
    '          return async function() {',
    '            var args = Array.prototype.slice.call(arguments);',
    '            var bridge = await ensureBridge(mapping.alias);',
    '            return bridge.executeAction(mapping.rawActionId, args);',
    '          };',
    '        },',
    '      });',
    '    };',
    '  };',
    '  var assertNoConflict = function(target, key, nextValue, alias, section) {',
    '    if (!Object.prototype.hasOwnProperty.call(target, key)) return;',
    '    if (stableStringify(target[key]) !== stableStringify(nextValue)) {',
    "      throw new Error('[ModuleFederationPlugin.rsc] ' + section + ' conflict for \"' + key + '\" while merging remote \"' + alias + '\".');",
    '    }',
    '  };',
    '  var getNamespacedModuleId = function(alias, rawId) {',
    "    return MODULE_PREFIX + alias + ':' + String(rawId);",
    '  };',
    '  var mergeRecord = function(target, source, alias, section) {',
    '    if (!isObject(source)) return;',
    '    for (var _i = 0, _entries = Object.entries(source); _i < _entries.length; _i++) {',
    '      var entry = _entries[_i];',
    '      var key = entry[0];',
    '      var value = entry[1];',
    '      assertNoConflict(target, key, value, alias, section);',
    '      target[key] = value;',
    '    }',
    '  };',
    '  var mergeRemoteManifest = function(alias, remoteManifest) {',
    '    if (!isObject(remoteManifest)) return;',
    '    var hostManifest = ensureHostManifest();',
    '    var namespacedClientIds = Object.create(null);',
    '    if (isObject(remoteManifest.clientManifest)) {',
    '      for (var _i = 0, _entries = Object.entries(remoteManifest.clientManifest); _i < _entries.length; _i++) {',
    '        var entry = _entries[_i];',
    '        var key = entry[0];',
    '        var value = entry[1];',
    '        var nextValue = isObject(value) ? Object.assign({}, value) : value;',
    '        if (isObject(nextValue) && nextValue.id != null) {',
    '          var namespacedClientId = getNamespacedModuleId(alias, nextValue.id);',
    '          namespacedClientIds[String(nextValue.id)] = namespacedClientId;',
    '          nextValue.id = namespacedClientId;',
    '        }',
    "        assertNoConflict(hostManifest.clientManifest, key, nextValue, alias, 'clientManifest');",
    '        hostManifest.clientManifest[key] = nextValue;',
    '      }',
    '    }',
    '    if (isObject(remoteManifest.serverConsumerModuleMap)) {',
    '      for (var _i = 0, _entries = Object.entries(remoteManifest.serverConsumerModuleMap); _i < _entries.length; _i++) {',
    '        var entry = _entries[_i];',
    '        var rawModuleId = entry[0];',
    '        var value = entry[1];',
    '        var scopedModuleId = Object.prototype.hasOwnProperty.call(namespacedClientIds, rawModuleId)',
    '          ? namespacedClientIds[rawModuleId]',
    '          : getNamespacedModuleId(alias, rawModuleId);',
    "        assertNoConflict(hostManifest.serverConsumerModuleMap, scopedModuleId, value, alias, 'serverConsumerModuleMap');",
    '        hostManifest.serverConsumerModuleMap[scopedModuleId] = value;',
    '      }',
    '    }',
    '    var remoteServerManifest = remoteManifest.serverManifest;',
    '    if (!isObject(remoteServerManifest)) return;',
    '    ensureActionProxyModule();',
    '    for (var _i = 0, _entries = Object.entries(remoteServerManifest); _i < _entries.length; _i++) {',
    '      var entry = _entries[_i];',
    '      var rawActionId = entry[0];',
    '      var actionEntry = entry[1];',
    "      var prefixedActionId = ACTION_PREFIX + alias + ':' + rawActionId;",
    '      var hostActionEntry = {',
    '        id: proxyModuleId,',
    '        name: prefixedActionId,',
    '        chunks: [],',
    '        async: actionEntry && actionEntry.async !== undefined ? actionEntry.async : true,',
    '      };',
    "      assertNoConflict(hostManifest.serverManifest, prefixedActionId, hostActionEntry, alias, 'serverManifest');",
    '      hostManifest.serverManifest[prefixedActionId] = hostActionEntry;',
    '      actionMap[prefixedActionId] = { alias: alias, rawActionId: rawActionId };',
    '    }',
    '  };',
    '  return function createRspackRscRuntimePlugin() {',
    '    return {',
    "      name: 'rspack-rsc-runtime-plugin',",
    '      onLoad: async function(args) {',
    '        var alias = (args && args.remote && args.remote.alias) || (args && args.pkgNameOrAlias) || (args && args.remote && args.remote.name);',
    '        if (!alias || mergedRemoteAliases.has(alias)) return args;',
    '        var expose = typeof (args && args.expose) === "string" ? args.expose : "";',
    '        if (expose.includes(RSC_BRIDGE_EXPOSE)) return args;',
    '        mergedRemoteAliases.add(alias);',
    '        try {',
    '          var bridge = await ensureBridge(alias);',
    '          mergeRemoteManifest(alias, bridge.getManifest());',
    '        } catch (error) {',
    '          mergedRemoteAliases.delete(alias);',
    '          throw error;',
    '        }',
    '        return args;',
    '      },',
    '    };',
    '  };',
    '}',
  ].join('\n');
}
