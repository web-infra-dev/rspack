import { defineConfig } from '@rspack/cli';
import { type Configuration, type CssParserExportType } from '@rspack/core';
import path from 'node:path';

const makeConfig = (
  exportType: CssParserExportType,
  useLess: boolean,
): Configuration => {
  const sourceFile = useLess ? 'style.less' : 'style.css';
  return {
    name: `${exportType}${useLess ? '-less' : ''}`,
    target: 'web',
    mode: 'development',
    devtool: 'source-map',
    resolve: {
      alias: {
        STYLE_UNDER_TEST$: path.resolve(import.meta.dirname, sourceFile),
      },
    },
    module: {
      rules: [
        useLess
          ? {
              test: /\.less$/,
              use: [
                {
                  loader: 'less-loader',
                  options: { sourceMap: true },
                },
              ],
              type: 'css/auto',
              ...(exportType !== 'link' && { parser: { exportType } }),
            }
          : {
              test: /\.css$/,
              type: 'css/auto',
              ...(exportType !== 'link' && { parser: { exportType } }),
            },
      ],
    },
  };
};

export default defineConfig([
  makeConfig('link', false),
  makeConfig('text', false),
  makeConfig('style', false),
  makeConfig('css-style-sheet', false),
  makeConfig('link', true),
  makeConfig('text', true),
  makeConfig('style', true),
  makeConfig('css-style-sheet', true),
]);
