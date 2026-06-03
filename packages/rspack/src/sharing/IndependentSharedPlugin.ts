import { join, resolve } from 'node:path';

import type { Compiler } from '../Compiler';
import type { LibraryOptions, Plugins, RspackOptions } from '../config';
import {
  getFileName,
  type ModuleFederationManifestPluginOptions,
} from '../container/ModuleFederationManifestPlugin';
import { parseOptions } from '../container/options';
import {
  CollectSharedEntryPlugin,
  type ShareRequestsMap,
} from './CollectSharedEntryPlugin';
import { ConsumeSharedPlugin } from './ConsumeSharedPlugin';
import {
  SharedContainerPlugin,
  type SharedContainerPluginOptions,
} from './SharedContainerPlugin';
import { SharedUsedExportsOptimizerPlugin } from './SharedUsedExportsOptimizerPlugin';
import type { Shared, SharedConfig } from './SharePlugin';
import { encodeName, isRequiredVersion } from './utils';

const VIRTUAL_ENTRY = './virtual-entry.js';
const VIRTUAL_ENTRY_NAME = 'virtual-entry';
const BUILD_SHARED_FALLBACK_STAGE = 102;

export type MakeRequired<T, K extends keyof T> = Required<Pick<T, K>> &
  Omit<T, K>;

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
  injectTreeShakingUsedExports?: boolean;
  treeShakingSharedExcludePlugins?: string[];
  onBuildAssets?: (buildAssets: ShareFallback) => void;
}

// { react: [  [ react/19.0.0/index.js , 19.0.0, react_global_name ]  ] }
export type ShareFallback = Record<string, [string, string, string][]>;

class VirtualEntryPlugin {
  sharedOptions: [string, SharedConfig][];
  collectShared = false;
  constructor(sharedOptions: [string, SharedConfig][], collectShared: boolean) {
    this.sharedOptions = sharedOptions;
    this.collectShared = collectShared;
  }
  createEntry() {
    const { sharedOptions, collectShared } = this;
    const entryContent = sharedOptions.reduce<string>((acc, cur, index) => {
      const importLine = `import shared_${index} from '${cur[0]}';\n`;
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

const resolveOutputDir = (outputDir: string, shareName?: string) => {
  return shareName ? join(outputDir, encodeName(shareName)) : outputDir;
};

const getShareRequests = (
  shareRequestsMap: ShareRequestsMap,
  shareName: string,
) =>
  Array.from(
    new Map(
      (shareRequestsMap[shareName]?.requests || []).map(
        ([request, version]) => [version, [request, version] as const],
      ),
    ).values(),
  );

export class IndependentSharedPlugin {
  mfName: string;
  shared: Shared;
  library?: LibraryOptions;
  sharedOptions: [string, SharedConfig][];
  outputDir: string;
  plugins: Plugins;
  treeShaking?: boolean;
  manifest?: ModuleFederationManifestPluginOptions;
  buildAssets: ShareFallback = {};
  injectTreeShakingUsedExports?: boolean;
  treeShakingSharedExcludePlugins?: string[];
  onBuildAssets?: (buildAssets: ShareFallback) => void;

  name = 'IndependentSharedPlugin';
  constructor(options: IndependentSharePluginOptions) {
    const {
      outputDir,
      plugins,
      treeShaking,
      shared,
      name,
      manifest,
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
    this.injectTreeShakingUsedExports = injectTreeShakingUsedExports ?? true;
    this.library = library;
    this.treeShakingSharedExcludePlugins =
      treeShakingSharedExcludePlugins || [];
    this.onBuildAssets = onBuildAssets;
    this.sharedOptions = parseOptions(
      shared,
      (item, key) => {
        if (typeof item !== 'string')
          throw new Error(
            `Unexpected array in shared configuration for key "${key}"`,
          );
        const config: SharedConfig =
          item === key || !isRequiredVersion(item)
            ? {
                import: item,
              }
            : {
                import: key,
                requiredVersion: item,
              };

        return config;
      },
      (item) => {
        return item;
      },
    );
  }

  apply(compiler: Compiler) {
    const { manifest } = this;
    const collectSharedEntryPlugin = new CollectSharedEntryPlugin({
      sharedOptions: this.sharedOptions,
      shareScope: 'default',
    });

    collectSharedEntryPlugin.apply(compiler);

    compiler.hooks.finishMake.tapPromise(
      {
        name: 'IndependentSharedPlugin',
        stage: BUILD_SHARED_FALLBACK_STAGE,
      },
      async () => {
        const shareRequestsMap = collectSharedEntryPlugin.getData();
        this.prepareBuildAssets(shareRequestsMap);
        await this.createIndependentCompilers(compiler, shareRequestsMap);
        this.onBuildAssets?.(this.buildAssets);
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
                    fallback?: string;
                    fallbackName?: string;
                  }[];
                };

                const { shared } = statsContent;
                Object.entries(this.buildAssets).forEach(([key, item]) => {
                  const targetShared = shared.find((s) => s.name === key);
                  if (!targetShared) {
                    return;
                  }
                  item.forEach(([entry, version, globalName]) => {
                    if (version === targetShared.version) {
                      targetShared.fallback = entry;
                      targetShared.fallbackName = globalName;
                    }
                  });
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

  private prepareBuildAssets(shareRequestsMap: ShareRequestsMap) {
    const { sharedOptions, outputDir, mfName, treeShaking, library } = this;
    const buildAssets: ShareFallback = {};

    sharedOptions.forEach(([shareName, shareConfig]) => {
      if (!shareConfig.treeShaking || shareConfig.import === false) {
        return;
      }
      const sharedConfig = sharedOptions.find(
        ([name]) => name === shareName,
      )?.[1];
      const shareRequests = getShareRequests(shareRequestsMap, shareName);

      shareRequests.forEach(([request, version]) => {
        const sharedContainerPlugin = new SharedContainerPlugin({
          mfName: `${mfName}_${treeShaking ? 't' : 'f'}`,
          library,
          shareName,
          version,
          request,
          independentShareFileName: sharedConfig?.treeShaking?.filename,
        });
        const [shareFileName, globalName, sharedVersion] =
          sharedContainerPlugin.getData();
        if (typeof shareFileName === 'string') {
          buildAssets[shareName] ||= [];
          buildAssets[shareName].push([
            join(resolveOutputDir(outputDir, shareName), shareFileName),
            sharedVersion,
            globalName,
          ]);
        }
      });
    });

    this.buildAssets = buildAssets;
  }

  private async createIndependentCompilers(
    parentCompiler: Compiler,
    shareRequestsMap: ShareRequestsMap,
  ) {
    const { sharedOptions } = this;
    console.log('Start building shared fallback resources ...');

    await Promise.all(
      sharedOptions.map(async ([shareName, shareConfig]) => {
        if (!shareConfig.treeShaking || shareConfig.import === false) {
          return;
        }
        const shareRequests = getShareRequests(shareRequestsMap, shareName);
        await Promise.all(
          shareRequests.map(async ([request, version]) => {
            const sharedConfig = sharedOptions.find(
              ([name]) => name === shareName,
            )?.[1];
            await this.createIndependentCompiler(parentCompiler, {
              shareRequestsMap,
              currentShare: {
                shareName,
                version,
                request,
                independentShareFileName: sharedConfig?.treeShaking?.filename,
              },
            });
          }),
        );
      }),
    );

    console.log('All shared fallback have been compiled successfully!');
  }

  private async createIndependentCompiler(
    parentCompiler: Compiler,
    extraOptions: {
      currentShare: Omit<SharedContainerPluginOptions, 'mfName'>;
      shareRequestsMap: ShareRequestsMap;
    },
  ) {
    const {
      mfName,
      plugins,
      outputDir,
      sharedOptions,
      treeShaking,
      library,
      treeShakingSharedExcludePlugins,
    } = this;

    const outputDirWithShareName = resolveOutputDir(
      outputDir,
      extraOptions.currentShare.shareName,
    );
    const parentConfig = parentCompiler.options;

    const finalPlugins = [];
    const rspack = parentCompiler.rspack;
    const extraPlugin = new SharedContainerPlugin({
      mfName: `${mfName}_${treeShaking ? 't' : 'f'}`,
      library,
      ...extraOptions.currentShare,
    });
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
          .filter(
            ([key, options]) =>
              extraOptions.currentShare.shareName !== (options.shareKey || key),
          )
          .map(([key, options]) => ({
            [key]: {
              import: false,
              shareKey: options.shareKey || key,
              shareScope: options.shareScope,
              requiredVersion: options.requiredVersion,
              strictVersion: options.strictVersion,
              singleton: options.singleton,
              packageName: options.packageName,
              eager: options.eager,
            },
          })),
        enhanced: true,
      }),
    );

    if (treeShaking) {
      finalPlugins.push(
        new SharedUsedExportsOptimizerPlugin(
          sharedOptions,
          this.injectTreeShakingUsedExports,
        ),
      );
    }
    finalPlugins.push(new VirtualEntryPlugin(sharedOptions, false));
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

    const { currentShare } = extraOptions;

    return new Promise<any>((resolve, reject) => {
      compiler.run((err: any, stats: any) => {
        if (err || stats?.hasErrors()) {
          console.error(
            `${currentShare.shareName} Compile failed:`,
            err ||
              stats
                .toJson()
                .errors.map((e: Error) => e.message)
                .join('\n'),
          );
          reject(err || new Error(`${currentShare.shareName} Compile failed`));
          return;
        }

        console.log(`${currentShare.shareName} Compile success`);
        resolve(extraPlugin.getData());
      });
    });
  }
}
