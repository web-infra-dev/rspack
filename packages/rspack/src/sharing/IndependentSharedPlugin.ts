import { join, posix, relative, resolve } from 'node:path';

import type { Compiler } from '../Compiler';
import type { LibraryOptions, Plugins, RspackOptions } from '../config';
import {
  getFileName,
  type ModuleFederationManifestPluginOptions,
} from '../container/ModuleFederationManifestPlugin';
import { createHash } from '../util/createHash';
import {
  CollectSharedEntryPlugin,
  type ShareRequestsMap,
} from './CollectSharedEntryPlugin';
import { ConsumeSharedPlugin } from './ConsumeSharedPlugin';
import { SharedContainerPlugin } from './SharedContainerPlugin';
import { SharedUsedExportsOptimizerPlugin } from './SharedUsedExportsOptimizerPlugin';
import {
  normalizeShareScope,
  normalizeSharedOptions,
  type ShareScope,
  type Shared,
  type SharedConfig,
} from './SharePlugin';
import {
  encodeName,
  resolveShareKey,
  resolveShareRequest,
  resolveShareScope,
} from './utils';

const VIRTUAL_ENTRY = './virtual-entry.js';
const VIRTUAL_ENTRY_NAME = 'virtual-entry';
const BUILD_SHARED_FALLBACK_STAGE = 102;

const filterPlugin = (plugin: Plugins[0], excludedPlugins: string[] = []) => {
  if (!plugin) {
    return true;
  }
  const pluginName = plugin.name || plugin.constructor?.name;
  if (!pluginName) {
    return true;
  }
  return ![
    'TreeShakingSharedPlugin',
    'IndependentSharedPlugin',
    'ModuleFederationPlugin',
    'SharedUsedExportsOptimizerPlugin',
    'HtmlWebpackPlugin',
    'HtmlRspackPlugin',
    'RsbuildHtmlPlugin',
    ...excludedPlugins,
  ].includes(pluginName);
};

export interface IndependentSharePluginOptions {
  name: string;
  shared: Shared;
  library?: LibraryOptions;
  outputDir?: string;
  plugins?: Plugins;
  treeShaking?: boolean;
  manifest?: ModuleFederationManifestPluginOptions;
  shareScope?: ShareScope;
  injectTreeShakingUsedExports?: boolean;
  treeShakingSharedExcludePlugins?: string[];
  onBuildAssets?: (
    buildAssets: ShareFallback,
    variants: ShareFallbackVariants,
  ) => void;
}

// { react: [  [ react/19.0.0/index.js , 19.0.0, react_global_name ]  ] }
export type ShareFallback = Record<string, [string, string, string][]>;

export type ShareFallbackVariant = {
  entry: string;
  version: string;
  globalName: string;
  shareScope: ShareScope;
  layer?: string;
  import: string;
};

export type ShareFallbackVariants = Record<string, ShareFallbackVariant[]>;

type SharedBuildRequest = {
  configIndex: number;
  configKey: string;
  shareKey: string;
  shareScope: ShareScope;
  layer?: string;
  issuerLayer?: string;
  configuredRequest: string;
  fallbackImport: string;
  request: string;
  version: string;
  independentShareFileName?: string;
  artifactIdentity?: string;
};

type SharedBuildAsset = SharedBuildRequest & {
  entry: string;
  globalName: string;
};

class VirtualEntryPlugin {
  requests: string[];
  collectShared = false;
  constructor(requests: string[], collectShared: boolean) {
    this.requests = requests;
    this.collectShared = collectShared;
  }
  createEntry() {
    const { requests, collectShared } = this;
    const entryContent = requests.reduce<string>((acc, request, index) => {
      const importLine = `import shared_${index} from ${JSON.stringify(request)};\n`;
      // Always mark the import as used to prevent tree-shaking removal
      // Optional console for debugging: reference the variable, not a string
      const logLine = collectShared ? `console.log(shared_${index});\n` : '';
      return acc + importLine + logLine;
    }, '');
    return entryContent;
  }

  static entry() {
    return {
      [VIRTUAL_ENTRY_NAME]: VIRTUAL_ENTRY,
    };
  }

  apply(compiler: Compiler) {
    new compiler.rspack.experiments.VirtualModulesPlugin({
      [VIRTUAL_ENTRY]: this.createEntry(),
    }).apply(compiler);

    compiler.hooks.thisCompilation.tap(
      'RemoveVirtualEntryAsset',
      (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'RemoveVirtualEntryAsset',
            stage: compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE,
          },
          () => {
            try {
              const chunk = compilation.namedChunks.get(VIRTUAL_ENTRY_NAME);

              chunk?.files.forEach((f) => {
                compilation.deleteAsset(f);
              });
            } catch (_e) {
              console.error('Failed to remove virtual entry file!');
            }
          },
        );
      },
    );
  }
}

const resolveOutputDir = (
  outputDir: string,
  shareName?: string,
  artifactIdentity?: string,
) => {
  if (!shareName) return outputDir;
  const shareOutputDir = join(outputDir, encodeName(shareName));
  return artifactIdentity
    ? join(shareOutputDir, `variant-${artifactIdentity}`)
    : shareOutputDir;
};

const resolvePublicOutputDir = (
  outputDir: string,
  shareName: string,
  artifactIdentity?: string,
) =>
  posix.join(
    outputDir.replaceAll('\\', '/'),
    encodeName(shareName),
    artifactIdentity ? `variant-${artifactIdentity}` : '',
  );

const toShareScopes = (shareScope: ShareScope): string[] =>
  Array.isArray(shareScope) ? shareScope : [shareScope];

const shareScopesEqual = (left: ShareScope, right: ShareScope) => {
  const leftScopes = toShareScopes(left);
  const rightScopes = toShareScopes(right);
  return (
    leftScopes.length === rightScopes.length &&
    leftScopes.every((scope, index) => scope === rightScopes[index])
  );
};

const createArtifactIdentity = (
  request: SharedBuildRequest,
  context: string,
) => {
  const hash = createHash('xxhash64');
  const resourceIdentity = relative(context, request.request).replaceAll(
    '\\',
    '/',
  );
  hash.update(
    Buffer.from(
      JSON.stringify([
        request.configKey,
        request.shareKey,
        toShareScopes(request.shareScope),
        request.layer ?? null,
        request.issuerLayer ?? null,
        request.configuredRequest,
        request.fallbackImport,
        resourceIdentity,
        request.version,
      ]),
    ),
  );
  return hash.digest('hex').slice(0, 12);
};

const getShareRequests = (
  shareRequestsMap: ShareRequestsMap,
  shareName: string,
  shareConfig: SharedConfig,
  rootShareScope: ShareScope = 'default',
) => {
  const entry =
    shareRequestsMap[resolveShareKey(shareConfig.shareKey, shareName)];
  const variants =
    entry?.variants ||
    (entry
      ? [
          {
            shareScope: entry.shareScope,
            layer: undefined,
            requests: entry.requests,
          },
        ]
      : []);
  const expectedScope = normalizeShareScope(
    resolveShareScope(shareConfig.shareScope, rootShareScope),
    true,
    'IndependentSharedPlugin',
  );
  const matchesScope = (shareScope: ShareScope) =>
    shareScopesEqual(shareScope, expectedScope);
  const exact = variants.filter(
    ({ layer, shareScope }) =>
      layer === shareConfig.layer && matchesScope(shareScope),
  );
  const selected =
    exact.length > 0 || shareConfig.layer === undefined
      ? exact
      : variants.filter(
          ({ layer, shareScope }) =>
            layer === undefined && matchesScope(shareScope),
        );
  const requests = selected.flatMap(({ requests }) => requests);
  return Array.from(
    new Map(
      requests.map(([request, version]) => [
        JSON.stringify([request, version]),
        [request, version] as const,
      ]),
    ).values(),
  );
};

export class IndependentSharedPlugin {
  mfName: string;
  shared: Shared;
  library?: LibraryOptions;
  sharedOptions: [string, SharedConfig][];
  outputDir: string;
  plugins: Plugins;
  treeShaking?: boolean;
  manifest?: ModuleFederationManifestPluginOptions;
  shareScope: ShareScope;
  buildAssets: ShareFallback = {};
  private buildAssetRecords: SharedBuildAsset[] = [];
  private buildAssetVariants: ShareFallbackVariants = {};
  injectTreeShakingUsedExports?: boolean;
  treeShakingSharedExcludePlugins?: string[];
  onBuildAssets?: (
    buildAssets: ShareFallback,
    variants: ShareFallbackVariants,
  ) => void;

  name = 'IndependentSharedPlugin';
  constructor(options: IndependentSharePluginOptions) {
    const {
      outputDir,
      plugins,
      treeShaking,
      shared,
      name,
      manifest,
      shareScope,
      injectTreeShakingUsedExports,
      library,
      treeShakingSharedExcludePlugins,
      onBuildAssets,
    } = options;
    this.shared = shared;
    this.mfName = name;
    this.outputDir = outputDir || 'independent-packages';
    this.plugins = plugins || [];
    this.treeShaking = treeShaking;
    this.manifest = manifest;
    this.shareScope = normalizeShareScope(
      shareScope || 'default',
      true,
      this.name,
    );
    this.injectTreeShakingUsedExports = injectTreeShakingUsedExports ?? true;
    this.library = library;
    this.treeShakingSharedExcludePlugins =
      treeShakingSharedExcludePlugins || [];
    this.onBuildAssets = onBuildAssets;
    this.sharedOptions = normalizeSharedOptions(shared);
  }

  apply(compiler: Compiler) {
    const { manifest } = this;
    const collectSharedEntryPlugin = new CollectSharedEntryPlugin({
      sharedOptions: this.sharedOptions,
      shareScope: this.shareScope,
    });

    collectSharedEntryPlugin.apply(compiler);

    compiler.hooks.finishMake.tapPromise(
      {
        name: 'IndependentSharedPlugin',
        stage: BUILD_SHARED_FALLBACK_STAGE,
      },
      async () => {
        const shareRequestsMap = collectSharedEntryPlugin.getData();
        const buildRequests = this.getBuildRequests(
          shareRequestsMap,
          compiler.context,
        );
        this.prepareBuildAssets(buildRequests);
        await this.createIndependentCompilers(compiler, buildRequests);
        this.onBuildAssets?.(this.buildAssets, this.buildAssetVariants);
      },
    );

    // inject buildAssets to stats
    if (manifest) {
      compiler.hooks.compilation.tap(
        'IndependentSharedPlugin',
        (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'injectBuildAssets',
              stage: (compilation.constructor as any)
                .PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER,
            },
            () => {
              const { statsFileName, manifestFileName } = getFileName(manifest);
              const injectBuildAssetsIntoStatsOrManifest = (
                filename: string,
              ) => {
                const stats = compilation.getAsset(filename);
                if (!stats) {
                  return;
                }
                const statsContent = JSON.parse(
                  stats.source.source().toString(),
                ) as {
                  shared: {
                    name: string;
                    version: string;
                    layer?: string;
                    shareScope?: ShareScope;
                    fallback?: string;
                    fallbackName?: string;
                  }[];
                };

                statsContent.shared.forEach((targetShared) => {
                  const candidates = this.buildAssetRecords.filter(
                    ({ shareKey, version, layer, shareScope }) =>
                      shareKey === targetShared.name &&
                      version === targetShared.version &&
                      (targetShared.layer === undefined ||
                        layer === targetShared.layer) &&
                      (targetShared.shareScope === undefined ||
                        shareScopesEqual(shareScope, targetShared.shareScope)),
                  );
                  if (candidates.length !== 1) return;
                  targetShared.fallback = candidates[0].entry;
                  targetShared.fallbackName = candidates[0].globalName;
                });

                compilation.updateAsset(
                  filename,
                  new compiler.rspack.sources.RawSource(
                    JSON.stringify(statsContent),
                  ),
                );
              };

              injectBuildAssetsIntoStatsOrManifest(statsFileName);
              injectBuildAssetsIntoStatsOrManifest(manifestFileName);
            },
          );
        },
      );
    }
  }

  private getBuildRequests(
    shareRequestsMap: ShareRequestsMap,
    context: string,
  ) {
    const buildRequests: SharedBuildRequest[] = [];

    this.sharedOptions.forEach(([configKey, shareConfig], configIndex) => {
      if (!shareConfig.treeShaking || shareConfig.import === false) return;
      const shareKey = resolveShareKey(shareConfig.shareKey, configKey);
      const shareScope = normalizeShareScope(
        resolveShareScope(shareConfig.shareScope, this.shareScope),
        true,
        this.name,
      );
      const configuredRequest = resolveShareRequest(
        shareConfig.request,
        configKey,
      );
      const fallbackImport =
        typeof shareConfig.import === 'string'
          ? shareConfig.import
          : configuredRequest;
      const requests = getShareRequests(
        shareRequestsMap,
        configKey,
        shareConfig,
        this.shareScope,
      );

      requests.forEach(([request, version]) => {
        buildRequests.push({
          configIndex,
          configKey,
          shareKey,
          shareScope,
          layer: shareConfig.layer,
          issuerLayer: shareConfig.issuerLayer,
          configuredRequest,
          fallbackImport,
          request,
          version,
          independentShareFileName: shareConfig.treeShaking?.filename,
        });
      });
    });

    const emittedPath = (request: SharedBuildRequest) =>
      JSON.stringify([
        resolvePublicOutputDir(this.outputDir, request.shareKey),
        (
          request.independentShareFileName ||
          `${request.version}/share-entry.js`
        ).replaceAll('\\', '/'),
      ]);
    const emittedGlobal = (request: SharedBuildRequest) =>
      encodeName(
        `${this.mfName}_${this.treeShaking ? 't' : 'f'}_${request.shareKey}_${request.version}`,
      );
    const pathCounts = new Map<string, number>();
    const globalCounts = new Map<string, number>();
    for (const request of buildRequests) {
      const path = emittedPath(request);
      const global = emittedGlobal(request);
      pathCounts.set(path, (pathCounts.get(path) || 0) + 1);
      globalCounts.set(global, (globalCounts.get(global) || 0) + 1);
    }
    for (const request of buildRequests) {
      if (
        request.layer !== undefined ||
        pathCounts.get(emittedPath(request))! > 1 ||
        globalCounts.get(emittedGlobal(request))! > 1
      ) {
        request.artifactIdentity = createArtifactIdentity(request, context);
      }
    }

    return buildRequests;
  }

  private prepareBuildAssets(buildRequests: SharedBuildRequest[]) {
    const { outputDir } = this;
    const buildAssets: ShareFallback = {};
    const buildAssetRecords: SharedBuildAsset[] = [];
    const buildAssetVariants: ShareFallbackVariants = {};

    buildRequests.forEach((request) => {
      const sharedContainerPlugin = this.createSharedContainerPlugin(request);
      const [shareFileName, globalName, sharedVersion] =
        sharedContainerPlugin.getData();
      if (typeof shareFileName !== 'string') return;
      const entry = posix.join(
        resolvePublicOutputDir(
          outputDir,
          request.shareKey,
          request.artifactIdentity,
        ),
        shareFileName.replaceAll('\\', '/'),
      );
      buildAssets[request.configKey] ||= [];
      buildAssets[request.configKey].push([entry, sharedVersion, globalName]);
      buildAssetVariants[request.shareKey] ||= [];
      buildAssetVariants[request.shareKey].push({
        entry,
        version: sharedVersion,
        globalName,
        shareScope: request.shareScope,
        layer: request.layer,
        import: request.fallbackImport,
      });
      buildAssetRecords.push({ ...request, entry, globalName });
    });

    this.buildAssets = buildAssets;
    this.buildAssetRecords = buildAssetRecords;
    this.buildAssetVariants = buildAssetVariants;
  }

  private createSharedContainerPlugin(request: SharedBuildRequest) {
    return new SharedContainerPlugin({
      mfName: `${this.mfName}_${this.treeShaking ? 't' : 'f'}`,
      library: this.library,
      shareName: request.shareKey,
      shareKey: request.shareKey,
      shareScope: request.shareScope,
      layer: request.layer,
      version: request.version,
      request: request.request,
      independentShareFileName: request.independentShareFileName,
      artifactIdentity: request.artifactIdentity,
    });
  }

  private async createIndependentCompilers(
    parentCompiler: Compiler,
    buildRequests: SharedBuildRequest[],
  ) {
    console.log('Start building shared fallback resources ...');

    await Promise.all(
      buildRequests.map((currentShare) =>
        this.createIndependentCompiler(parentCompiler, currentShare),
      ),
    );

    console.log('All shared fallback have been compiled successfully!');
  }

  private async createIndependentCompiler(
    parentCompiler: Compiler,
    currentShare: SharedBuildRequest,
  ) {
    const {
      plugins,
      outputDir,
      sharedOptions,
      treeShaking,
      treeShakingSharedExcludePlugins,
    } = this;

    const outputDirWithShareName = resolveOutputDir(
      outputDir,
      currentShare.shareKey,
      currentShare.artifactIdentity,
    );
    const parentConfig = parentCompiler.options;

    const finalPlugins = [];
    const rspack = parentCompiler.rspack;
    const extraPlugin = this.createSharedContainerPlugin(currentShare);
    (parentConfig.plugins || []).forEach((plugin) => {
      if (
        plugin !== undefined &&
        typeof plugin !== 'string' &&
        filterPlugin(plugin, treeShakingSharedExcludePlugins)
      ) {
        finalPlugins.push(plugin);
      }
    });
    plugins.forEach((plugin) => {
      finalPlugins.push(plugin);
    });
    finalPlugins.push(extraPlugin);

    finalPlugins.push(
      new ConsumeSharedPlugin({
        consumes: sharedOptions
          .filter((_, index) => index !== currentShare.configIndex)
          .map(([key, options]) => ({
            [key]: {
              import: false,
              shareKey: resolveShareKey(options.shareKey, key),
              shareScope: options.shareScope,
              requiredVersion: options.requiredVersion,
              strictVersion: options.strictVersion,
              singleton: options.singleton,
              packageName: options.packageName,
              eager: options.eager,
              issuerLayer: options.issuerLayer,
              layer: options.layer,
              request: resolveShareRequest(options.request, key),
            },
          })),
        shareScope: this.shareScope,
        enhanced: true,
      }),
    );

    if (treeShaking) {
      finalPlugins.push(
        new SharedUsedExportsOptimizerPlugin(
          sharedOptions,
          this.injectTreeShakingUsedExports,
          undefined,
          this.shareScope,
        ),
      );
    }
    finalPlugins.push(
      new VirtualEntryPlugin(
        sharedOptions.map(([key, options], index) =>
          index === currentShare.configIndex
            ? currentShare.request
            : resolveShareRequest(options.request, key),
        ),
        false,
      ),
    );
    const fullOutputDir = resolve(
      parentCompiler.outputPath,
      outputDirWithShareName,
    );
    const compilerConfig: RspackOptions = {
      ...parentConfig,
      name: parentConfig.name || 'mf-shared-compiler',
      module: {
        ...parentConfig.module,
        rules: [
          {
            test: /virtual-entry\.js$/,
            type: 'javascript/auto',
            resolve: { fullySpecified: false },
            use: {
              loader: 'builtin:swc-loader',
            },
          },
          ...(parentConfig.module?.rules || []),
        ],
      },
      mode: parentConfig.mode || 'development',

      entry: VirtualEntryPlugin.entry,

      output: {
        path: fullOutputDir,
        clean: false,
        publicPath: parentConfig.output?.publicPath || 'auto',
      },

      plugins: finalPlugins,

      optimization: {
        ...parentConfig.optimization,
        splitChunks: false,
      },
    };

    const compiler = rspack.rspack(compilerConfig);

    compiler.inputFileSystem = parentCompiler.inputFileSystem;
    compiler.outputFileSystem = parentCompiler.outputFileSystem;
    compiler.intermediateFileSystem = parentCompiler.intermediateFileSystem;

    return new Promise<any>((resolve, reject) => {
      compiler.run((err: any, stats: any) => {
        if (err || stats?.hasErrors()) {
          console.error(
            `${currentShare.shareKey} Compile failed:`,
            err ||
              stats
                .toJson()
                .errors.map((e: Error) => e.message)
                .join('\n'),
          );
          reject(err || new Error(`${currentShare.shareKey} Compile failed`));
          return;
        }

        console.log(`${currentShare.shareKey} Compile success`);
        resolve(extraPlugin.getData());
      });
    });
  }
}
