/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/IgnoreWarningsPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import util from 'node:util';
import type {
  Compiler,
  IgnoreWarningsNormalized,
  RspackPluginInstance,
} from '..';

class IgnoreWarningsPlugin implements RspackPluginInstance {
  _ignorePattern: IgnoreWarningsNormalized;
  name = 'IgnoreWarningsPlugin';

  /**
   * @param ignoreWarnings conditions to ignore warnings
   */
  constructor(ignorePattern: IgnoreWarningsNormalized) {
    this._ignorePattern = ignorePattern;
  }

  /**
   * Apply the plugin
   * @param compiler the compiler instance
   * @returns
   */
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(this.name, (compilation) => {
      compilation.hooks.processWarnings.tap(this.name, (warnings) => {
        return warnings.filter((warning) => {
          // VT control characters are stripped to avoid false mismatches
          // caused by terminal coloring or formatting.
          const plainWarning = warning.message
            ? {
                ...warning,
                message: util.stripVTControlCharacters(warning.message),
              }
            : warning;
          return !this._ignorePattern.some((ignore) =>
            ignore(plainWarning, compilation),
          );
        });
      });
    });
  }
}

export default IgnoreWarningsPlugin;
