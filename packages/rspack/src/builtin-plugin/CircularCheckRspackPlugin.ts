import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawCircularCheckRspackPluginOptions,
} from '@rspack/binding';
import type { Compilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import type { Module } from '../Module';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export type CircularCheckRspackPluginOptions = {
  /**
   * Exclude detection of files based on a RegExp.
   */
  exclude?: RegExp;
  /**
   * Include specific files based on a RegExp.
   */
  include?: RegExp;
  /**
   * Add errors to rspack instead of warnings.
   */
  failOnError?: boolean;
  /**
   * Called for each detected circular dependency.
   */
  onDetected?(args: {
    module: Module;
    paths: string[];
    compilation: Compilation;
  }): void;
};

export class CircularCheckRspackPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.CircularCheckRspackPlugin;
  _options: CircularCheckRspackPluginOptions;

  constructor(options: CircularCheckRspackPluginOptions = {}) {
    super();
    this._options = options;
  }

  raw(compiler: Compiler): BuiltinPlugin {
    const { failOnError, exclude, include, onDetected } = this._options;

    const rawOptions: RawCircularCheckRspackPluginOptions = {
      failOnError,
      exclude,
      include,
      onDetected: onDetected
        ? (module: Module, paths: string[]) => {
            const compilation: Compilation =
              compiler.__internal__get_compilation()!;
            onDetected({ module, paths, compilation });
          }
        : undefined,
    };

    return createBuiltinPlugin(this.name, rawOptions);
  }
}
