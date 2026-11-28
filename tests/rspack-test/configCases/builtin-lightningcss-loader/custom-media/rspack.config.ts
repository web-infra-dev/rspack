import {
  type Compiler,
  type LightningcssLoaderOptions,
  type Module,
  NormalModule,
} from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class AssertModuleSourcePlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'AssertModuleSourcePlugin',
      (compilation) => {
        let seen = false;
        const checkModule = (module: Module) => {
          if (
            !(module instanceof NormalModule) ||
            !module.resource.endsWith('index.module.css')
          ) {
            return;
          }
          seen = true;
          // Module sizes, and therefore chunk splitting, are based on this source.
          const source = module.originalSource()?.source().toString();
          expect(source).toBeDefined();
          expect(source).not.toContain('@custom-media');
          expect(source).not.toContain('--wide');
        };
        compilation.hooks.succeedModule.tap(
          'AssertModuleSourcePlugin',
          checkModule,
        );
        compilation.hooks.stillValidModule.tap(
          'AssertModuleSourcePlugin',
          checkModule,
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
