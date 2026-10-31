import {
  type Compiler,
  type LightningcssLoaderOptions,
  NormalModule,
} from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class AssertModuleSourcePlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'AssertModuleSourcePlugin',
      (compilation) => {
        let seen = false;
        compilation.hooks.succeedModule.tap(
          'AssertModuleSourcePlugin',
          (module) => {
            if (
              !(module instanceof NormalModule) ||
              !module.resource.endsWith('index.module.css')
            ) {
              return;
            }
            seen = true;
            // Module sizes, and therefore chunk splitting, are based on this source.
            const source = module.originalSource()?.source().toString();
            // A failed build has no source; the loader assertion reports why.
            if (source === undefined) return;
            expect(source).not.toContain('@custom-media');
            expect(source).not.toContain('--wide');
          },
        );
        compilation.hooks.finishModules.tap('AssertModuleSourcePlugin', () => {
          expect(seen).toBe(true);
        });
      },
    );
  }
}

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  devtool: 'source-map',
  module: {
    parser: {
      'css/auto': {
        namedExports: true,
      },
    },
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        use: [
          './assert-loader-output.mjs',
          {
            loader: 'builtin:lightningcss-loader',
            options: {
              minify: false,
              drafts: {
                customMedia: true,
              },
              include: {
                customMediaQueries: true,
              },
            } satisfies LightningcssLoaderOptions,
          },
        ],
      },
    ],
  },
  plugins: [new AssertModuleSourcePlugin()],
});
