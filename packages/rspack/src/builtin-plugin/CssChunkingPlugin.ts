import binding from '@rspack/binding';

import { create } from './base';

export interface CssChunkingPluginOptions {
  strict?: boolean;
  minSize?: number;
  maxSize?: number;
  /**
   * This plugin is intended to be generic, but currently requires some special handling for Next.js.
   * A `next` option has been added to accommodate this.
   * In the future, once the design of CssChunkingPlugin becomes more stable, this option may be removed.
   */
  nextjs?: boolean;
}

export const CssChunkingPlugin = create(
  binding.BuiltinPluginName.CssChunkingPlugin,
  function (
    options: CssChunkingPluginOptions = {},
  ): binding.RawCssChunkingPluginOptions {
    if (options.nextjs) {
      return {
        strict: options.strict,
        minSize: options.minSize,
        maxSize: options.maxSize,
        exclude: /^pages\//,
      };
    }
    const { splitChunks } = this.options.optimization;

    if (splitChunks) {
      const cssMiniExtractIndex =
        splitChunks.defaultSizeTypes!.indexOf('css/mini-extract');
      if (cssMiniExtractIndex) {
        splitChunks.defaultSizeTypes!.splice(cssMiniExtractIndex, 1);
      }
      const cssIndex = splitChunks.defaultSizeTypes!.indexOf('css');
      if (cssIndex) {
        splitChunks.defaultSizeTypes!.splice(cssIndex, 1);
      }
    }
    return options;
  },
);
