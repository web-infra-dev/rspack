/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import type { Callback } from '@rspack/lite-tapable';
import { Compiler } from './Compiler';
import {
  applyRspackOptionsBaseDefaults,
  applyRspackOptionsDefaults,
  getNormalizedRspackOptions,
  type RspackOptions,
  type RspackPluginFunction,
} from './config';
import {
  MultiCompiler,
  type MultiCompilerOptions,
  type MultiRspackOptions,
} from './MultiCompiler';
import MultiStats from './MultiStats';
import NodeEnvironmentPlugin from './node/NodeEnvironmentPlugin';
import { RspackOptionsApply } from './rspackOptionsApply';
import { Stats } from './Stats';
import { deprecate, isNil } from './util';
import { validateRspackConfig } from './util/validateConfig';

function createMultiCompiler(options: MultiRspackOptions): MultiCompiler {
  const compilers = options.map(createCompiler);
  const compiler = new MultiCompiler(
    compilers,
    options as MultiCompilerOptions,
  );
  for (const childCompiler of compilers) {
    if (childCompiler.options.dependencies) {
      compiler.setDependencies(
        childCompiler,
        childCompiler.options.dependencies,
      );
    }
  }

  return compiler;
}

function createCompiler(userOptions: RspackOptions): Compiler {
  const options = getNormalizedRspackOptions(userOptions);
  applyRspackOptionsBaseDefaults(options);

  if (isNil(options.context)) {
    throw new Error('options.context is required');
  }

  const compiler = new Compiler(options.context, options);

  new NodeEnvironmentPlugin({
    infrastructureLogging: options.infrastructureLogging,
  }).apply(compiler);

  if (Array.isArray(options.plugins)) {
    for (const plugin of options.plugins) {
      if (typeof plugin === 'function') {
        (plugin as RspackPluginFunction).call(compiler, compiler);
      } else if (plugin) {
        plugin.apply(compiler);
      }
    }
  }

  const tp = applyRspackOptionsDefaults(compiler.options);
  if (tp) {
    compiler.platform = tp.platform;
    compiler.target = { esVersion: tp.esVersion, targets: tp.targets };
  }

  compiler.hooks.environment.call();
  compiler.hooks.afterEnvironment.call();
  new RspackOptionsApply().process(compiler.options, compiler);
  compiler.hooks.initialize.call();
  return compiler;
}

function isMultiRspackOptions(o: unknown): o is MultiRspackOptions {
  return Array.isArray(o);
}

function rspack(options: MultiRspackOptions): MultiCompiler;
function rspack(options: RspackOptions): Compiler;
function rspack(
  options: MultiRspackOptions | RspackOptions,
): MultiCompiler | Compiler;
function rspack(
  options: MultiRspackOptions,
  callback?: Callback<Error, MultiStats>,
): null | MultiCompiler;
function rspack(
  options: RspackOptions,
  callback?: Callback<Error, Stats>,
): null | Compiler;
function rspack(
  options: MultiRspackOptions | RspackOptions,
  callback?: Callback<Error, MultiStats | Stats>,
): null | MultiCompiler | Compiler;
function rspack(
  options: MultiRspackOptions | RspackOptions,
  callback?: Callback<Error, MultiStats> | Callback<Error, Stats>,
) {
  try {
    if (isMultiRspackOptions(options)) {
      for (const option of options) {
        validateRspackConfig(option);
      }
    } else {
      validateRspackConfig(options);
    }
  } catch (err) {
    if (err instanceof Error && callback) {
      callback(err);
      return null;
    }
    throw err;
  }

  const create = () => {
    if (isMultiRspackOptions(options)) {
      const compiler = createMultiCompiler(options);
      const watch = options.some((options) => options.watch);
      const watchOptions = options.map((options) => options.watchOptions || {});
      return { compiler, watch, watchOptions };
    }
    const compiler = createCompiler(options);
    const watch = options.watch;
    const watchOptions = options.watchOptions || {};
    return { compiler, watch, watchOptions };
  };

  if (callback) {
    try {
      const { compiler, watch, watchOptions } = create();
      if (watch) {
        compiler.watch(watchOptions, callback as any);
      } else {
        compiler.run((err, stats) => {
          compiler.close(() => {
            callback(err, stats as any);
          });
        });
      }
      return compiler;
    } catch (err: any) {
      process.nextTick(() => callback(err));
      return null;
    }
  } else {
    const { compiler, watch } = create();
    if (watch) {
      deprecate(
        "A 'callback' argument needs to be provided to the 'rspack(options, callback)' function when the 'watch' option is set. There is no way to handle the 'watch' option without a callback.",
      );
    }
    return compiler;
  }
}

// deliberately alias rspack as webpack
export { createCompiler, createMultiCompiler, MultiStats, rspack, Stats };
export default rspack;
