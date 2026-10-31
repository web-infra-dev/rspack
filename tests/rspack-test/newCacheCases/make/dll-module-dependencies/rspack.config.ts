import fs from 'node:fs';
import path from 'node:path';
import { DllPlugin } from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

const manifestPath = path.join(import.meta.dirname, 'manifest.json');
let buildIndex = 0;
let built: string[] = [];
let reused: string[] = [];

export default defineConfig({
  entry: { main: ['./index.js'] },
  target: 'node',
  mode: 'development',
  devtool: false,
  incremental: false,
  cache: { type: 'persistent' },
  experiments: {
    newCache: {
      module: true,
      loader: false,
      codeGeneration: false,
      devtool: false,
      minimize: false,
    },
  },
  externals: { external: 'commonjs external' },
  output: {
    filename: 'dll.js',
    library: { type: 'commonjs2' },
  },
  optimization: { concatenateModules: false, minimize: false },
  plugins: [
    new DllPlugin({ path: manifestPath }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'PolymorphicModuleCache',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'PolymorphicModuleCache',
              (module) => {
                built.push(module.identifier());
              },
            );
            compilation.hooks.stillValidModule.tap(
              'PolymorphicModuleCache',
              (module) => {
                reused.push(module.identifier());
              },
            );
          },
        );
        compiler.hooks.done.tap('PolymorphicModuleCache', (stats) => {
          expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
          // DllModule overrides need_build to reuse its build; ExternalModule keeps
          // the conservative default and must rebuild even when an entry exists.
          expect(built.filter((id) => id === 'dll main')).toHaveLength(
            buildIndex === 0 ? 1 : 0,
          );
          expect(reused.filter((id) => id === 'dll main')).toHaveLength(
            buildIndex === 0 ? 0 : 1,
          );
          expect(built.filter((id) => id.startsWith('external '))).toHaveLength(
            1,
          );
          expect(
            reused.filter((id) => id.startsWith('external ')),
          ).toHaveLength(0);
          expect(built.filter((id) => /[\\/]index\.js$/.test(id))).toHaveLength(
            buildIndex === 0 ? 1 : 0,
          );

          const dllModule = [...stats.compilation.modules].find(
            (module) => module.identifier() === 'dll main',
          );
          expect(dllModule).toBeDefined();
          expect(dllModule!.dependencies).toHaveLength(1);
          const entryModule = stats.compilation.moduleGraph.getModule(
            dllModule!.dependencies[0],
          );
          expect(
            (
              entryModule as { resource?: string } | undefined
            )?.resource?.replace(/^.*[\\/]/, ''),
          ).toBe('index.js');

          const manifest = JSON.parse(
            fs.readFileSync(manifestPath, 'utf8'),
          ) as { content: Record<string, { id: string | number }> };
          const output: { exports: unknown } = { exports: {} };
          new Function(
            'require',
            'module',
            'exports',
            String(stats.compilation.getAsset('dll.js')!.source.source()),
          )(
            (request: unknown) => {
              expect(request).toBe('external');
              return 'external-';
            },
            output,
            output.exports,
          );
          expect(
            (output.exports as (id: string | number) => string)(
              manifest.content['./index.js'].id,
            ),
          ).toBe('external-initial');

          built = [];
          reused = [];
          buildIndex++;
        });
      },
    }),
  ],
});
