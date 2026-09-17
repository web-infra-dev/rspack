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
      publicPath: 'http://localhost:3000/',
      chunkFilename: '[name].0.js',
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
                data.assets.js.push('//localhost:3000/chunk.0.js');
                data.assets.js.push('http://localhost:3000/chunk.0.js');
                data.assets.js.push('//rspack.dev/chunk.0.js');
                data.assets.js.push('http://rspack.dev/chunk.0.js');
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
              /<script crossorigin defer integrity=".+" src="\/\/localhost:3000\/chunk\.0\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script crossorigin defer integrity=".+" src="http:\/\/localhost:3000\/chunk\.0\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="\/\/rspack.dev\/chunk\.0\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/rspack.dev\/chunk\.0\.js">/,
            );
          });
        },
      },
    ],
  },
  {
    target: 'web',
    output: {
      publicPath: 'http://localhost:3000/',
      chunkFilename: '[name].1.js',
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
                data.assets.js.push('//localhost:3000/chunk.1.js');
                data.assets.js.push('http://localhost:3000/chunk.1.js');
                data.assets.js.push('//rspack.dev/chunk.1.js');
                data.assets.js.push('http://rspack.dev/chunk.1.js');
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
              /<script defer src="\/\/localhost:3000\/chunk\.1\.js" integrity=".+" crossorigin="anonymous">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/localhost:3000\/chunk\.1\.js" integrity=".+" crossorigin="anonymous">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="\/\/rspack.dev\/chunk\.1\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/rspack.dev\/chunk\.1\.js">/,
            );
          });
        },
      },
    ],
  },
  {
    target: 'web',
    output: {
      publicPath: '//localhost:3000/',
      chunkFilename: '[name].2.js',
      crossOriginLoading: 'anonymous',
    },
    plugins: [
      new SubresourceIntegrityPlugin(),
      new BuiltinHtmlRspackPlugin({
        filename: 'index2.html',
      }),
      {
        apply(compiler) {
          compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
            BuiltinHtmlRspackPlugin.getCompilationHooks(
              compilation,
            ).beforeAssetTagGeneration.tap(
              'SubresourceIntegrityPlugin',
              (data) => {
                data.assets.js.push('//localhost:3000/chunk.2.js');
                data.assets.js.push('http://localhost:3000/chunk.2.js');
                data.assets.js.push('//rspack.dev/chunk.2.js');
                data.assets.js.push('http://rspack.dev/chunk.2.js');
              },
            );
          });
        },
      },
      {
        apply(compiler) {
          compiler.hooks.done.tap('TestPlugin', () => {
            const htmlContent = fs.readFileSync(
              path.resolve(testPath, 'index2.html'),
              'utf-8',
            );
            expect(htmlContent).toMatch(
              /<script crossorigin defer integrity=".+" src="\/\/localhost:3000\/chunk\.2\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script crossorigin defer integrity=".+" src="http:\/\/localhost:3000\/chunk\.2\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="\/\/rspack.dev\/chunk\.2\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/rspack.dev\/chunk\.2\.js">/,
            );
          });
        },
      },
    ],
  },
  {
    target: 'web',
    output: {
      publicPath: '//localhost:3000/',
      chunkFilename: '[name].3.js',
      crossOriginLoading: 'anonymous',
    },
    plugins: [
      new SubresourceIntegrityPlugin({
        htmlPlugin: fileURLToPath(import.meta.resolve('html-rspack-plugin')),
      }),
      new HtmlRspackPlugin({
        filename: 'index3.html',
      }),
      {
        apply(compiler) {
          compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
            HtmlRspackPlugin.getCompilationHooks(
              compilation,
            ).beforeAssetTagGeneration.tap(
              'SubresourceIntegrityPlugin',
              (data) => {
                data.assets.js.push('//localhost:3000/chunk.3.js');
                data.assets.js.push('http://localhost:3000/chunk.3.js');
                data.assets.js.push('//rspack.dev/chunk.3.js');
                data.assets.js.push('http://rspack.dev/chunk.3.js');
              },
            );
          });
        },
      },
      {
        apply(compiler) {
          compiler.hooks.done.tap('TestPlugin', () => {
            const htmlContent = fs.readFileSync(
              path.resolve(testPath, 'index3.html'),
              'utf-8',
            );
            expect(htmlContent).toMatch(
              /<script defer src="\/\/localhost:3000\/chunk\.3\.js" integrity=".+" crossorigin="anonymous">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/localhost:3000\/chunk\.3\.js" integrity=".+" crossorigin="anonymous">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="\/\/rspack.dev\/chunk\.3\.js">/,
            );
            expect(htmlContent).toMatch(
              /<script defer src="http:\/\/rspack.dev\/chunk\.3\.js">/,
            );
          });
        },
      },
    ],
  },
];
