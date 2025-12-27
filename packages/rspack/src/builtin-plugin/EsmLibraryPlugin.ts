import { BuiltinPluginName } from "@rspack/binding";
import rspack from "..";
import type { Compiler } from "../Compiler";
import type {
  OptimizationSplitChunksOptions,
  RspackOptionsNormalized
} from "../config";
import type { Logger } from "../logging/Logger";
import { RemoveDuplicateModulesPlugin } from "./RemoveDuplicateModulesPlugin";
import { toRawSplitChunksOptions } from "./SplitChunksPlugin";

const NOT_SUPPORTED_CONFIG = [
  "maxSize",
  "minSizeReduction",
  "maxAsyncSize",
  "maxInitialSize",
  "maxAsyncRequests",
  "maxInitialRequests"
] as const;

function applyLimits(options: RspackOptionsNormalized, logger: Logger) {
  // concatenateModules is not supported in ESM library mode, it has its own scope hoist algorithm
  options.optimization.concatenateModules = false;

  // esm library won't have useless empty chunk, the empty chunk for esm lib is to re-exports
  options.optimization.removeEmptyChunks = false;

  // chunk rendering is handled by EsmLibraryPlugin
  options.output.chunkFormat = false;

  // mark output is module
  options.output.module = true;

  if (options.output.chunkLoading && options.output.chunkLoading !== 'import') {
    logger.warn(
      `\`output.chunkLoading\` should be \`"import"\` or \`false\`, but got ${options.output.chunkLoading}, changed it to \`"import"\``,
    );
    options.output.chunkLoading = 'import';
  }

  if (options.output.chunkLoading === undefined) {
    options.output.chunkLoading = 'import';
  }

  if (options.output.library) {
    options.output.library = undefined;
  }

  let { splitChunks } = options.optimization;
  if (splitChunks === undefined) {
    splitChunks = options.optimization.splitChunks = false;
  }

  if (splitChunks !== false) {
    const notSupportedConfig = ['maxSize', 'minSizeReduction', 'maxAsyncSize', 'maxInitialSize', 'maxAsyncRequests', 'maxInitialRequests'] as const;
    const invalidConfig = [];

    for (const field of notSupportedConfig) {
      if (fieldUsed(field, splitChunks)) {
        invalidConfig.push(field);
      }
    }

    if (invalidConfig.length > 0) {
      logger.warn(
        `Currently \`${invalidConfig.join(", ")}\` are not supported in esm library mode`
      );
    }

    splitChunks.cacheGroups ??= {};
    splitChunks.cacheGroups.default = false;
    splitChunks.cacheGroups.defaultVendors = false;
  }
}

function fieldUsed(
  field: (typeof NOT_SUPPORTED_CONFIG)[number],
  splitChunks: OptimizationSplitChunksOptions
): boolean {
  if (splitChunks[field] !== undefined) {
    return true;
  }

  const cacheGroups = splitChunks.cacheGroups;
  if (cacheGroups === undefined) {
    return false;
  }

  for (const key of Reflect.ownKeys(cacheGroups)) {
    const cacheGroup = cacheGroups[key as string];
    if (!cacheGroup) {
      continue;
    }

    if (cacheGroup[field] !== undefined) {
      return true;
    }
  }

  return false;
}

export class EsmLibraryPlugin {
  static PLUGIN_NAME = "EsmLibraryPlugin";
  options?: {
    preserveModules?: string;
    splitChunks?: OptimizationSplitChunksOptions;
  };

  constructor(options?: { preserveModules?: string }) {
    this.options = options;
  }

  apply(compiler: Compiler) {
    const logger = compiler.getInfrastructureLogger(
      EsmLibraryPlugin.PLUGIN_NAME,
    );
    applyLimits(compiler.options, logger);

    if (compiler.options.optimization.splitChunks) {
      this.options!.splitChunks = compiler.options.optimization.splitChunks;

      compiler.options.optimization.splitChunks = false;
    }

    new RemoveDuplicateModulesPlugin().apply(compiler);

    let err;
    if ((err = checkConfig(compiler.options))) {
      throw new rspack.WebpackError(
        `Conflicted config for ${EsmLibraryPlugin.PLUGIN_NAME}: ${err}`,
      );
    }

    compiler.__internal__registerBuiltinPlugin({
      name: BuiltinPluginName.EsmLibraryPlugin,
      options: {
        preserveModules: this.options?.preserveModules,
        splitChunks:
          this.options?.splitChunks &&
          toRawSplitChunksOptions(this.options?.splitChunks, compiler)
      }
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
