import path from 'node:path';
import util from 'node:util';
import type {
  Compiler,
  MultiCompiler,
  MultiRspackOptions,
  MultiStats,
  RspackOptions,
  RspackPluginFunction,
  RspackPluginInstance,
  Stats,
} from '@rspack/core';
import cac, { type CAC } from 'cac';
import { createColors, isColorSupported } from 'picocolors';
import { BuildCommand } from './commands/build';
import { PreviewCommand } from './commands/preview';
import { ServeCommand } from './commands/serve';
import type { RspackCLIColors, RspackCLILogger } from './types';
import { loadExtendedConfig, loadRspackConfig } from './utils/loadConfig';
import type {
  CommonOptions,
  CommonOptionsForBuildAndServe,
} from './utils/options';
import { rspack } from './utils/rspackCore';

type Command = 'serve' | 'build';

declare global {
  const RSPACK_CLI_VERSION: string;
}

export class RspackCLI {
  colors: RspackCLIColors;
  program: CAC;

  constructor() {
    const program = cac('rspack');
    this.colors = this.createColors();
    this.program = program;
    program.help();
    program.version(RSPACK_CLI_VERSION);
  }

  async createCompiler(
    options: CommonOptionsForBuildAndServe,
    rspackCommand: Command,
    callback?: (e: Error | null, res?: Stats | MultiStats) => void,
  ) {
    let { config, pathMap } = await this.loadConfig(options);
    config = await this.buildConfig(config, pathMap, options, rspackCommand);

    const isWatch = Array.isArray(config)
      ? (config as MultiRspackOptions).some((i) => i.watch)
      : (config as RspackOptions).watch;

    let compiler: MultiCompiler | Compiler | null;
    try {
      compiler = rspack(config, isWatch ? callback : undefined);
      if (!isWatch && compiler) {
        // unsafeFastDrop is an internal option api and not shown in types
        compiler.unsafeFastDrop = true;
      }
    } catch (e) {
      // Aligned with webpack-cli
      // See: https://github.com/webpack/webpack-cli/blob/eea6adf7d34dfbfd3b5b784ece4a4664834f5a6a/packages/webpack-cli/src/webpack-cli.ts#L2394
      if (e instanceof rspack.ValidationError) {
        this.getLogger().error(e.message);
        process.exit(2);
      } else if (e instanceof Error) {
        if (typeof callback === 'function') {
          callback(e);
        } else {
          this.getLogger().error(e);
        }
        return null;
      }
      throw e;
    }
    return compiler;
  }

  createColors(useColor?: boolean): RspackCLIColors {
    const shouldUseColor = useColor || isColorSupported;
    return {
      ...createColors(shouldUseColor),
      isColorSupported: shouldUseColor,
    };
  }

  getLogger(): RspackCLILogger {
    return {
      error: (val) =>
        console.error(`[rspack-cli] ${this.colors.red(util.format(val))}`),
      warn: (val) => console.warn(`[rspack-cli] ${this.colors.yellow(val)}`),
      info: (val) => console.info(`[rspack-cli] ${this.colors.cyan(val)}`),
      success: (val) => console.log(`[rspack-cli] ${this.colors.green(val)}`),
      log: (val) => console.log(`[rspack-cli] ${val}`),
      raw: (val) => console.log(val),
    };
  }

  async run(argv: string[]) {
    await this.registerCommands();
    this.program.parse(argv);
  }

  async registerCommands() {
    const builtinCommands = [
      new BuildCommand(),
      new ServeCommand(),
      new PreviewCommand(),
    ];
    for (const command of builtinCommands) {
      await command.apply(this);
    }
  }
  async buildConfig(
    item: RspackOptions | MultiRspackOptions,
    pathMap: WeakMap<RspackOptions, string[]>,
    options: CommonOptionsForBuildAndServe,
    command: Command,
  ): Promise<RspackOptions | MultiRspackOptions> {
    const isBuild = command === 'build';
    const isServe = command === 'serve';

    const internalBuildConfig = async (item: RspackOptions) => {
      if (options.entry) {
        item.entry = {
          main: options.entry.map((x) => path.resolve(process.cwd(), x))[0], // Fix me when entry supports array
        };
      }
      // to set output.path
      item.output = item.output || {};
      if (options.outputPath) {
        item.output.path = path.resolve(process.cwd(), options.outputPath);
      }
      if (options.analyze) {
        const { BundleAnalyzerPlugin } =
          await import('webpack-bundle-analyzer');
        (item.plugins ??= []).push({
          name: 'rspack-bundle-analyzer',
          apply(compiler: any) {
            new BundleAnalyzerPlugin({
              generateStatsFile: true,
            }).apply(compiler);
          },
        });
      }
      if (options.profile) {
        item.profile = true;
      }
      if (process.env.RSPACK_PROFILE) {
        const { applyProfile } = await import('./utils/profile.js');
        await applyProfile(
          process.env.RSPACK_PROFILE,
          process.env.RSPACK_TRACE_LAYER,
          process.env.RSPACK_TRACE_OUTPUT,
        );
      }
      // cli --watch overrides the watch config
      if (options.watch) {
        item.watch = options.watch;
      }
      // auto set default mode if user config don't set it
      if (!item.mode) {
        item.mode = isBuild ? 'production' : 'development';
      }
      // user parameters always has highest priority than default mode and config mode
      if (options.mode) {
        item.mode = options.mode as RspackOptions['mode'];
      }

      // false is also a valid value for sourcemap, so don't override it
      if (typeof item.devtool === 'undefined') {
        item.devtool = isBuild ? 'source-map' : 'cheap-module-source-map';
      }
      // The CLI flag has a higher priority than the default devtool and devtool from the config.
      if (typeof options.devtool !== 'undefined') {
        item.devtool = options.devtool as RspackOptions['devtool'];
      }

      if (isServe) {
        const installed = (item.plugins ||= []).find(
          (item) => item instanceof rspack.ProgressPlugin,
        );
        if (!installed) {
          (item.plugins ??= []).push(new rspack.ProgressPlugin());
        }
      }

      // set configPaths to persistent cache build dependencies
      const cacheOptions = item.experiments?.cache;
      if (
        typeof cacheOptions === 'object' &&
        cacheOptions.type === 'persistent'
      ) {
        const configPaths = pathMap.get(item);
        if (configPaths) {
          // for persistent cache
          cacheOptions.buildDependencies = [
            ...configPaths,
            ...(cacheOptions.buildDependencies || []),
          ];
        }
      }

      if (typeof item.stats === 'undefined') {
        item.stats = { preset: 'errors-warnings', timings: true };
      } else if (typeof item.stats === 'boolean') {
        item.stats = item.stats ? { preset: 'normal' } : { preset: 'none' };
      } else if (typeof item.stats === 'string') {
        item.stats = {
          preset: item.stats as
            | 'normal'
            | 'none'
            | 'verbose'
            | 'errors-only'
            | 'errors-warnings',
        };
      }
      if (
        this.colors.isColorSupported &&
        typeof item.stats.colors === 'undefined'
      ) {
        item.stats.colors = true;
      }
      return item;
    };

    if (Array.isArray(item)) {
      return Promise.all(item.map(internalBuildConfig));
    }
    return internalBuildConfig(item as RspackOptions);
  }

  async loadConfig(options: CommonOptions): Promise<{
    config: RspackOptions | MultiRspackOptions;
    pathMap: WeakMap<RspackOptions, string[]>;
  }> {
    const config = await loadRspackConfig(options);
    // can not found any config
    if (!config) {
      return {
        config: this.filterConfig(options, {}),
        pathMap: new WeakMap(),
      };
    }

    let { loadedConfig, configPath } = config;

    if (typeof loadedConfig === 'function') {
      let functionResult = loadedConfig(
        options.env as Record<string, unknown>,
        options,
      );
      // if return promise we should await its result
      if (
        typeof (functionResult as unknown as Promise<unknown>).then ===
        'function'
      ) {
        functionResult = await functionResult;
      }

      loadedConfig = functionResult;
    }

    // Handle extends property if the loaded config is not a function
    const { config: extendedConfig, pathMap } = await loadExtendedConfig(
      loadedConfig as RspackOptions | MultiRspackOptions,
      configPath,
      process.cwd(),
      options,
    );

    return {
      config: this.filterConfig(options, extendedConfig),
      pathMap,
    };
  }

  private filterConfig(
    options: CommonOptions,
    config: RspackOptions | MultiRspackOptions,
  ): RspackOptions | MultiRspackOptions {
    if (options.configName) {
      const notFoundConfigNames: string[] = [];

      config = options.configName.map((configName: string) => {
        let found: RspackOptions | undefined;

        if (Array.isArray(config)) {
          found = config.find((options) => options.name === configName);
        } else {
          found =
            (config as RspackOptions).name === configName
              ? (config as RspackOptions)
              : undefined;
        }

        if (!found) {
          notFoundConfigNames.push(configName);
        }

        // WARNING: if config is not found, the program will exit
        // so assert here is okay to avoid runtime filtering
        return found!;
      });

      if (notFoundConfigNames.length > 0) {
        this.getLogger().error(
          notFoundConfigNames
            .map(
              (configName) =>
                `Configuration with the name "${configName}" was not found.`,
            )
            .join(' '),
        );
        process.exit(2);
      }
    }
    return config;
  }

  isMultipleCompiler(
    compiler: Compiler | MultiCompiler,
  ): compiler is MultiCompiler {
    return Boolean((compiler as MultiCompiler).compilers);
  }
  isWatch(compiler: Compiler | MultiCompiler): boolean {
    return Boolean(
      this.isMultipleCompiler(compiler)
        ? compiler.compilers.some((compiler) => compiler.options.watch)
        : compiler.options.watch,
    );
  }
}

export type RspackConfigFn = (
  env: Record<string, any>,
  argv: Record<string, any>,
) => RspackOptions | MultiRspackOptions;

export type RspackConfigAsyncFn = (
  env: Record<string, any>,
  argv: Record<string, any>,
) => Promise<RspackOptions | MultiRspackOptions>;

export type RspackConfigExport =
  | RspackOptions
  | MultiRspackOptions
  | RspackConfigFn
  | RspackConfigAsyncFn;

/**
 * This function helps you to autocomplete configuration types.
 * It accepts a Rspack config object, or a function that returns a config.
 */
export function defineConfig(config: RspackOptions): RspackOptions;
export function defineConfig(config: MultiRspackOptions): MultiRspackOptions;
export function defineConfig(config: RspackConfigFn): RspackConfigFn;
export function defineConfig(config: RspackConfigAsyncFn): RspackConfigAsyncFn;
export function defineConfig(config: RspackConfigExport): RspackConfigExport;
export function defineConfig(config: RspackConfigExport) {
  return config;
}

// Note: use union type will make apply function's `compiler` type to be `any`
export function definePlugin(
  plugin: RspackPluginFunction,
): RspackPluginFunction;
export function definePlugin(
  plugin: RspackPluginInstance,
): RspackPluginInstance;
export function definePlugin(plugin: any): any {
  return plugin;
}
