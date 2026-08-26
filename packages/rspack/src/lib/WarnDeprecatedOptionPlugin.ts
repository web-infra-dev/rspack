/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/WarnDeprecatedOptionPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compiler, RspackPluginInstance } from '..';
import WebpackError from './WebpackError';

class DeprecatedOptionWarning extends WebpackError {
  constructor(option: string, value: string | number, suggestion: string) {
    super();

    this.name = 'DeprecatedOptionWarning';
    this.message =
      'configuration\n' +
      `The value '${value}' for option '${option}' is deprecated. ` +
      `Use '${suggestion}' instead.`;
  }
}

class WarnDeprecatedOptionPlugin implements RspackPluginInstance {
  option: string;
  value: string | number;
  suggestion: string;
  name = 'WarnDeprecatedOptionPlugin';

  /**
   * @param option the target option
   * @param value the deprecated option value
   * @param suggestion the suggestion replacement
   */
  constructor(option: string, value: string | number, suggestion: string) {
    this.option = option;
    this.value = value;
    this.suggestion = suggestion;
  }

  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap(this.name, (compilation) => {
      compilation.warnings.push(
        new DeprecatedOptionWarning(this.option, this.value, this.suggestion),
      );
    });
  }
}

export default WarnDeprecatedOptionPlugin;
