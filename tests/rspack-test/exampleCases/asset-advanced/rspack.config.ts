import { defineConfig } from '@rspack/cli';
import svgToMiniDataURI from 'mini-svg-data-uri';

export default defineConfig({
  output: {
    assetModuleFilename: 'images/[hash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg)$/,
        type: 'asset',
      },
      {
        test: /\.svg$/,
        type: 'asset',
        generator: {
          dataUrl: (content: string | Buffer) => {
            if (typeof content !== 'string') {
              content = content.toString();
            }

            return svgToMiniDataURI(content);
          },
        },
      },
    ],
  },
});
