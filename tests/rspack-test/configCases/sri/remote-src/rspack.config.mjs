import {
  SubresourceIntegrityPlugin,
  HtmlRspackPlugin as BuiltinHtmlRspackPlugin,
} from '@rspack/core';
import HtmlRspackPlugin from 'html-rspack-plugin';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default (_, { testPath }) => [
  {
    target: 'web',
    output: {
      crossOriginLoading: 'anonymous',
    },
    plugins: [
      new SubresourceIntegrityPlugin(),
      new BuiltinHtmlRspackPlugin({
        filename: 'index.html',
      }),
      {
        apply(compiler) {
          compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
            BuiltinHtmlRspackPlugin.getCompilationHooks(
              compilation,
            ).beforeAssetTagGeneration.tap(
              'SubresourceIntegrityPlugin',
              (data) => {
                data.assets.js.push('http://localhost:3000/index.js');
              },
            );
          });
        },
      },
      {
        apply(compiler) {
          compiler.hooks.done.tap('TestPlugin', () => {
            const htmlContent = fs.readFileSync(
              path.resolve(testPath, 'index.html'),
              'utf-8',
            );
            expect(htmlContent).toMatch(
              /<script crossorigin defer integrity=".+" src="bundle0\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/localhost:3000\/index\.js">/,
            );
          });
        },
      },
    ],
  },
  {
    target: 'web',
    output: {
      crossOriginLoading: 'anonymous',
    },
    plugins: [
      new SubresourceIntegrityPlugin({
        htmlPlugin: fileURLToPath(import.meta.resolve('html-rspack-plugin')),
      }),
      new HtmlRspackPlugin({
        filename: 'index1.html',
      }),
      {
        apply(compiler) {
          compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
            HtmlRspackPlugin.getCompilationHooks(
              compilation,
            ).beforeAssetTagGeneration.tap(
              'SubresourceIntegrityPlugin',
              (data) => {
                data.assets.js.push('http://localhost:3000/index.js');
              },
            );
          });
        },
      },
      {
        apply(compiler) {
          compiler.hooks.done.tap('TestPlugin', () => {
            const htmlContent = fs.readFileSync(
              path.resolve(testPath, 'index1.html'),
              'utf-8',
            );
            expect(htmlContent).toMatch(
              /<script defer src="bundle1\.js" integrity=".+" crossorigin="anonymous">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/localhost:3000\/index\.js">/,
            );
          });
        },
      },
    ],
  },
];
