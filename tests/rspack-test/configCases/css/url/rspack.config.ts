import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack, type RspackPluginInstance } from '@rspack/core';

const checkFragmentDependencies: RspackPluginInstance = {
  apply(compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.finishModules.tap('Test', (modules) => {
        let encodedHashDependencies = 0;
        for (const module of modules) {
          for (const dependency of module.dependencies) {
            if (dependency.type === 'css url') {
              expect(dependency.request?.startsWith('#')).toBe(false);
              if (dependency.request?.includes('#icon.svg')) {
                encodedHashDependencies++;
                expect(
                  compilation.moduleGraph.getModule(dependency)?.type,
                ).toBe('asset/resource');
              }
            }
          }
        }
        expect(encodedHashDependencies).toBe(4);
      });
    });
  },
};

export default defineConfig([
  {
    target: 'web',
    mode: 'development',
    devtool: false,

    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css',
        },
        {
          test: /#icon\.svg$/,
          type: 'asset/resource',
          generator: {
            filename: 'encoded-[contenthash][ext][fragment]',
          },
        },
      ],
    },
    output: {
      assetModuleFilename: '[name].[hash][ext][query][fragment]',
    },
    resolve: {
      roots: [import.meta.dirname],
      alias: {
        'alias-url.png': path.resolve(import.meta.dirname, 'img.png'),
        'alias-url-1.png': false,
      },
    },
    externals: {
      'external-url.png': 'asset ./img.png',
      'external-url-2.png': 'test',
      'schema:test': "asset 'img.png'",
    },
    plugins: [
      new rspack.IgnorePlugin({ resourceRegExp: /ignore\.png/ }),
      checkFragmentDependencies,
    ],
  },
  {
    target: 'web',
    mode: 'development',
    devtool: false,

    module: {
      parser: {
        css: {
          url: false,
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css',
        },
      ],
    },
    output: {
      assetModuleFilename: '[name].[hash][ext][query][fragment]',
    },
  },
]);
