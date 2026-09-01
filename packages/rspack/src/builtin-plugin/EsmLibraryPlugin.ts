import { BuiltinPluginName } from '@rspack/binding';
import type { Compiler } from '../Compiler';
import type {
  OptimizationSplitChunksOptions,
  RspackOptionsNormalized,
} from '../config';
import WebpackError from '../lib/WebpackError';
import type { Logger } from '../logging/Logger';
import { RemoveDuplicateModulesPlugin } from './RemoveDuplicateModulesPlugin';
import { toRawSplitChunksOptions } from './SplitChunksPlugin';

export function applyLimits(options: RspackOptionsNormalized, logger?: Logger) {
  // concatenateModules is not supported in ESM library mode, it has its own scope hoist algorithm
  options.optimization.concatenateModules = false;

  // esm library won't have useless empty chunk, the empty chunk for esm lib is to re-exports
  options.optimization.removeEmptyChunks = false;

  // chunk rendering is handled by EsmLibraryPlugin
  options.output.chunkFormat = false;

  // mark output is module
  options.output.module = true;

  if (options.output.chunkLoading && options.output.chunkLoading !== 'import') {
    options.output.chunkLoading = 'import';
  }

  if (options.output.chunkLoading === undefined) {
    options.output.chunkLoading = 'import';
  }

  // Eval devtools wrap module code in eval(), while modern-module renders it at
  // the top level. Delay this fallback until options apply so it can warn.
  if (logger && typeof options.devtool === 'string') {
    const previousDevtool = options.devtool;
    let fallbackDevtool: RspackOptionsNormalized['devtool'];

    if (previousDevtool === 'eval') {
      fallbackDevtool = false;
    } else if (
      previousDevtool.startsWith('eval-') &&
      previousDevtool.includes('source-map')
    ) {
      fallbackDevtool = previousDevtool.slice(
        'eval-'.length,
      ) as RspackOptionsNormalized['devtool'];
    }

    if (fallbackDevtool !== undefined) {
      options.devtool = fallbackDevtool;
      logger.warn(
        `\`devtool: "${previousDevtool}"\` is not supported with \`library.type: "modern-module"\` because its modules are rendered as top-level code. Rspack has changed \`devtool\` to \`${JSON.stringify(fallbackDevtool)}\`.`,
      );
    }
  }

  let { splitChunks } = options.optimization;
  if (splitChunks === undefined) {
    splitChunks = options.optimization.splitChunks = {};
  }

  if (splitChunks !== false) {
    splitChunks.chunks = 'all';
    splitChunks.minSize = 0;
    splitChunks.maxAsyncRequests = Infinity;
    splitChunks.maxInitialRequests = Infinity;
    splitChunks.cacheGroups ??= {};
    splitChunks.cacheGroups.default = false;
    splitChunks.cacheGroups.defaultVendors = false;
  }
}

export class EsmLibraryPlugin {
  static PLUGIN_NAME = 'EsmLibraryPlugin';
  options: {
    preserveModules?: string;
    splitChunks?: OptimizationSplitChunksOptions | false;
  };

  constructor(options?: {
    preserveModules?: string;
    splitChunks?: OptimizationSplitChunksOptions | false;
  }) {
    this.options = options ?? {};
  }

  apply(compiler: Compiler) {
    applyLimits(compiler.options);

    new RemoveDuplicateModulesPlugin().apply(compiler);

    let err;
    if ((err = checkConfig(compiler.options))) {
      throw new WebpackError(
        `Conflicted config for ${EsmLibraryPlugin.PLUGIN_NAME}: ${err}`,
      );
    }

    compiler.__internal__registerBuiltinPlugin({
      name: BuiltinPluginName.EsmLibraryPlugin,
      options: {
        preserveModules: this.options.preserveModules,
        splitChunks: toRawSplitChunksOptions(
          this.options.splitChunks ?? false,
          compiler,
        ),
      },
    });
  }
}

function checkConfig(config: RspackOptionsNormalized): string | undefined {
  if (config.optimization.concatenateModules) {
    return 'You should disable `config.optimization.concatenateModules`';
  }

  if (config.output.chunkFormat !== false) {
    return 'You should disable default chunkFormat by `config.output.chunkFormat = false`';
  }
}
