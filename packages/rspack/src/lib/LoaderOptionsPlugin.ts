/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/LoaderOptionsPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compiler } from '../Compiler';
import { NormalModule } from '../NormalModule';
import type { MatchObject } from './ModuleFilenameHelpers';
import * as ModuleFilenameHelpers from './ModuleFilenameHelpers';

type LoaderOptionsPluginOptions = MatchObject & {
  [key: string]: unknown;
};

export class LoaderOptionsPlugin {
  options: LoaderOptionsPluginOptions;

  /**
   * @param options options object
   */
  constructor(options: LoaderOptionsPluginOptions = {}) {
    if (!options.test) {
      options.test = {
        test: () => true,
      } as unknown as MatchObject['test'];
    }
    this.options = options;
  }

  /**
   * Apply the plugin
   * @param compiler the compiler instance
   * @returns
   */
  apply(compiler: Compiler): void {
    const options = this.options;
    compiler.hooks.compilation.tap('LoaderOptionsPlugin', (compilation) => {
      NormalModule.getCompilationHooks(compilation).loader.tap(
        'LoaderOptionsPlugin',
        (context) => {
          const resource = context.resourcePath;
          if (!resource) return;
          if (ModuleFilenameHelpers.matchObject(options, resource)) {
            for (const key of Object.keys(options)) {
              if (key === 'include' || key === 'exclude' || key === 'test') {
                continue;
              }
              (context as any)[key] = options[key];
            }
          }
        },
      );
    });
  }
}
