import { defineConfig } from '@rspack/cli';
import type { CssParserExportType } from '@rspack/core';

/**
 * @param exportType the CSS parser exportType under test
 * @returns webpack configuration
 */
const makeConfig = (exportType: CssParserExportType) =>
  defineConfig({
    name: exportType,
    target: 'web',
    mode: 'development',
    devtool: 'nosources-source-map',
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
          ...(exportType !== 'link' && { parser: { exportType } }),
        },
      ],
    },
    experiments: {
      css: true,
    },
  });

export default defineConfig([
  makeConfig('link'),
  makeConfig('text'),
  makeConfig('style'),
  makeConfig('css-style-sheet'),
]);
