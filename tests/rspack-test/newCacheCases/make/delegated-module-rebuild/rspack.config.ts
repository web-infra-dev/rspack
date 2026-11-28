import {
  DllReferencePlugin,
  type DllReferencePluginOptionsContent,
} from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

const dllName = 'recovery-dll';
const sourceRequest = `dll-reference ${dllName}`;
const content: DllReferencePluginOptionsContent = {
  './numeric.js': {
    id: 7,
    exports: ['default', 'unused', 'value'],
    buildMeta: { exportsType: 'namespace', strictEsmModule: true, esm: true },
  },
  './named.js': {
    id: 'named-export',
    exports: ['default', 'value'],
    buildMeta: { exportsType: 'namespace', strictEsmModule: true, esm: true },
  },
};
const identifier = (id: number | string | undefined) =>
  `delegated ${id} from ${sourceRequest}`;
const expectedIdentifiers = Object.values(content)
  .map((entry) => identifier(entry.id))
  .sort();

let buildIndex = 0;
let built: string[] = [];
let reused: string[] = [];

export default defineConfig({
  entry: './index.js',
  target: 'node',
  mode: 'production',
  devtool: false,
  incremental: false,
  cache: false,
  experiments: {
    newCache: {
      module: true,
      loader: false,
      codeGeneration: false,
      devtool: false,
      minimize: false,
    },
  },
  output: {
    filename: 'main.js',
    library: { type: 'commonjs2' },
  },
  optimization: {
    concatenateModules: false,
    providedExports: true,
    minimize: false,
  },
  plugins: [
    new DllReferencePlugin({
      scope: 'dll',
      sourceType: 'commonjs2',
      manifest: { name: dllName, content },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'DelegatedModuleCache',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'DelegatedModuleCache',
              (module) => {
                if (module.identifier().startsWith('delegated ')) {
                  built.push(module.identifier());
                }
              },
            );
            compilation.hooks.stillValidModule.tap(
              'DelegatedModuleCache',
              (module) => {
                if (module.identifier().startsWith('delegated ')) {
                  reused.push(module.identifier());
                }
              },
            );
          },
        );
        compiler.hooks.done.tap('DelegatedModuleCache', (stats) => {
          // Persistent caching is disabled: every build must rebuild the delegated modules.
          expect(built.sort()).toEqual(expectedIdentifiers);
          expect(reused).toEqual([]);

          const { compilation } = stats;
          const modules = [...compilation.modules].filter((module) =>
            module.identifier().startsWith('delegated '),
          );
          expect(modules.map((module) => module.identifier()).sort()).toEqual(
            expectedIdentifiers,
          );
          for (const module of modules) {
            const manifestEntry = Object.values(content).find(
              (entry) => identifier(entry.id) === module.identifier(),
            )!;
            expect(
              module.dependencies.map((dependency) => dependency.type),
            ).toEqual(['delegated source', 'static exports']);
            expect(module.dependencies[0].request).toBe(sourceRequest);
            const sourceModule = compilation.moduleGraph.getModule(
              module.dependencies[0],
            );
            expect(sourceModule?.identifier()).toMatch(/^external /);
            expect(
              (
                compilation.moduleGraph.getProvidedExports(module) as string[]
              ).sort(),
            ).toEqual([...(manifestEntry.exports as string[])].sort());
          }

          const dllExports: Record<string | number, object> = {
            7: {
              default: 'numeric-default',
              value: 'numeric-value',
              unused: true,
            },
            'named-export': { default: 'named-default', value: 'named-value' },
          };
          const output = { exports: {} };
          new Function(
            'require',
            'module',
            'exports',
            String(compilation.getAsset('main.js')!.source.source()),
          )(
            (request: unknown) => {
              expect(request).toBe(dllName);
              return (id: string | number) => dllExports[id];
            },
            output,
            output.exports,
          );
          expect((output.exports as Record<string, unknown>).default).toEqual([
            'numeric-default',
            'named-default',
            'numeric-value',
            'named-value',
          ]);

          built = [];
          reused = [];
          buildIndex++;
        });
      },
    }),
  ],
});
