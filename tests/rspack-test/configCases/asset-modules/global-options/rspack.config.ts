import { defineConfig } from '@rspack/cli';
import svgToMiniDataURI from 'mini-svg-data-uri';
import mimeTypes from 'mime-types';

export default defineConfig({
  mode: 'development',
  module: {
    parser: {
      asset: {
        dataUrlCondition: (
          _source: Buffer,
          { filename }: { filename: string },
        ) => {
          return filename.includes('?inline');
        },
      },
    },
    generator: {
      asset: {
        dataUrl: (source, { module }) => {
          const mimeType = mimeTypes.lookup(module.nameForCondition()!);
          if (mimeType === 'image/svg+xml') {
            return svgToMiniDataURI(source.toString());
          }

          const encodedContent = source.toString('base64');

          return `DATA:${mimeType};base64,${encodedContent}`;
        },
      },
    },
    rules: [
      {
        test: /\.(png|svg)$/,
        type: 'asset',
      },
      {
        test: /\.jpg$/,
        type: 'asset/inline',
      },
    ],
  },
});
