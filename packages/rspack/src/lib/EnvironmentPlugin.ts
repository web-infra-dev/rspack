/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/EnvironmentPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compiler } from '../Compiler';
import { DefinePlugin } from '../builtin-plugin';
import WebpackError from './WebpackError';

// Waiting to adapt > import("./DefinePlugin").CodeValue
type CodeValue = any;

class EnvironmentPlugin {
  keys: string[];
  defaultValues: Record<string, string | undefined | null>;

  constructor(
    ...keys:
      string[] | [Record<string, string | undefined | null> | string | string[]]
  ) {
    if (keys.length === 1 && Array.isArray(keys[0])) {
      this.keys = keys[0];
      this.defaultValues = {};
    } else if (keys.length === 1 && keys[0] && typeof keys[0] === 'object') {
      this.keys = Object.keys(keys[0]);
      this.defaultValues = keys[0] as Record<string, string | undefined | null>;
    } else {
      this.keys = keys as string[];
      this.defaultValues = {};
    }
  }

  /**
   * Apply the plugin
   * @param compiler the compiler instance
   * @returns
   */
  apply(compiler: Compiler) {
    const definitions: Record<string, CodeValue> = Object.create(null);
    for (const key of this.keys) {
      const value = Object.prototype.hasOwnProperty.call(process.env, key)
        ? process.env[key]
        : this.defaultValues[key];

      if (value === undefined) {
        compiler.hooks.thisCompilation.tap(
          'EnvironmentPlugin',
          (compilation) => {
            const error = new WebpackError(
              `EnvironmentPlugin - ${key} environment variable is undefined.\n\n` +
                'You can pass an object with default values to suppress this warning.\n' +
                'See https://rspack.rs/plugins/webpack/environment-plugin for example.',
            );

            error.name = 'EnvVariableNotDefinedError';
            compilation.errors.push(error);
          },
        );
      }

      const defValue =
        value === undefined ? 'undefined' : JSON.stringify(value);
      definitions[`process.env.${key}`] = defValue;
      if (compiler.options.experiments.env) {
        definitions[`import.meta.env.${key}`] = defValue;
      }
    }
    new DefinePlugin(definitions).apply(compiler);
  }
}

export { EnvironmentPlugin };
