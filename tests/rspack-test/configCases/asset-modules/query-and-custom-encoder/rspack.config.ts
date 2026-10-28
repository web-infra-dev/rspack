import { defineConfig } from '@rspack/cli';
import type { AssetInlineGeneratorOptions } from '@rspack/core';
import svgToMiniDataURI from 'mini-svg-data-uri';
import mimeTypes from 'mime-types';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset/inline',
        generator: {
          dataUrl: (source, { filename, module }) => {
            if (filename.endsWith('?foo=bar')) {
              return svgToMiniDataURI(source.toString());
            }

            const mimeType = mimeTypes.lookup(module.nameForCondition()!);
            const encodedContent = source.toString('base64');

            return `data:${mimeType};base64,${encodedContent}`;
          },
        } satisfies AssetInlineGeneratorOptions,
      },
    ],
  },
});
