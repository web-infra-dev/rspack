import { defineConfig } from '@rspack/cli';
import type { AssetInlineGeneratorOptions } from '@rspack/core';
import svgToMiniDataURI from 'mini-svg-data-uri';
import mimeTypes from 'mime-types';

const generator: AssetInlineGeneratorOptions = {
  dataUrl: (source, { filename, module }) => {
    if (filename.endsWith('?foo=bar')) {
      return svgToMiniDataURI(source.toString());
    }

    const mimeType = mimeTypes.lookup(module.nameForCondition()!);
    const encodedContent = source.toString('base64');

    return `data:${mimeType};base64,${encodedContent}`;
  },
};

const rule = { test: /\.(png|svg|jpg)$/, type: 'asset/inline' };

export default defineConfig([
  {
    mode: 'development',
    module: { rules: [{ ...rule, generator }] },
  },
  {
    mode: 'development',
    module: {
      generator: { 'asset/inline': generator },
      rules: [rule],
    },
  },
]);
