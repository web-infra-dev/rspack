import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { type Compiler, NormalModule } from '@rspack/core';

const buildInfoKeys = [
  'assets',
  'fileDependencies',
  'contextDependencies',
  'missingDependencies',
  'buildDependencies',
].map((name) => Symbol.for(`rspack.buildInfo.${name}`));
const commitKey = Symbol.for('rspack.buildInfo.commitCustomFields');

export default defineConfig({
  module: {
    rules: [
      { test: /\.js$/, use: path.join(import.meta.dirname, 'loader.cjs') },
    ],
  },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.compilation.tap('SharedAccessors', (compilation) => {
        compilation.hooks.finishModules.tap('SharedAccessors', (modules) => {
          const normalModules = [...modules].filter(
            (m): m is NormalModule => m instanceof NormalModule,
          );
          expect(normalModules).toHaveLength(2);
          const [first, second] = normalModules;
          for (const module of normalModules) {
            const info = module.buildInfo;
            expect(info).toBe(module.buildInfo);
            expect(Object.keys(info)).toEqual(['resource']);
            expect(Reflect.ownKeys(info)).toEqual([
              'resource',
              ...buildInfoKeys,
              commitKey,
            ]);
            expect(info.resource).toBe(module.resource);
            expect(info.fileDependencies.has(module.resource)).toBe(true);
            expect(info.contextDependencies.has(import.meta.dirname)).toBe(
              true,
            );
            expect(
              info.missingDependencies.has(`${module.resource}.missing`),
            ).toBe(true);
            expect(
              info.buildDependencies.has(
                path.join(import.meta.dirname, 'loader.cjs'),
              ),
            ).toBe(true);
            expect(Object.keys(info.assets)).toEqual([
              module.resource.endsWith('index.js') ? 'index.txt' : 'value.txt',
            ]);
            for (const key of buildInfoKeys) {
              expect(Object.hasOwn(info, key)).toBe(true);
              const descriptor = Object.getOwnPropertyDescriptor(info, key)!;
              expect(descriptor.enumerable).toBe(false);
              expect(descriptor.configurable).toBe(true);
              expect(descriptor.set).toBeUndefined();
              expect(descriptor.get).toBe(
                Object.getOwnPropertyDescriptor(first.buildInfo, key)!.get,
              );
              expect(() => descriptor.get!.call({})).toThrow();
            }
            const commit = Object.getOwnPropertyDescriptor(info, commitKey)!;
            expect(commit.enumerable).toBe(false);
            expect(commit.configurable).toBe(true);
            expect(commit.writable).toBe(false);
            expect(commit.value).toBe(
              Object.getOwnPropertyDescriptor(first.buildInfo, commitKey)!
                .value,
            );
          }
          const moduleKeys = [
            'context',
            'layer',
            'useSourceMap',
            'useSimpleSourceMap',
            'factoryMeta',
            'buildInfo',
          ];
          for (const module of normalModules) {
            expect(Object.keys(module).slice(0, 16)).toEqual([
              'resource',
              'request',
              'userRequest',
              'rawRequest',
              'resourceResolveData',
              'loaders',
              'matchResource',
              'error',
              'type',
              ...moduleKeys,
              'buildMeta',
            ]);
            for (const key of moduleKeys) {
              expect(Object.hasOwn(module, key)).toBe(true);
              const descriptor = Object.getOwnPropertyDescriptor(module, key)!;
              const firstDescriptor = Object.getOwnPropertyDescriptor(
                first,
                key,
              )!;
              expect(descriptor.enumerable).toBe(true);
              expect(descriptor.configurable).toBe(true);
              expect(descriptor.get).toBe(firstDescriptor.get);
              expect(descriptor.set).toBe(firstDescriptor.set);
              expect(typeof descriptor.set).toBe(
                key === 'factoryMeta' || key === 'buildInfo'
                  ? 'function'
                  : 'undefined',
              );
              expect(() => descriptor.get!.call({})).toThrow();
            }
            expect(module.context).toBe(import.meta.dirname);
            expect(module.factoryMeta.sideEffectFree).toBe(false);
            expect(Reflect.set(module, 'context', 'ignored')).toBe(false);
            expect(() => {
              module.factoryMeta = {};
            }).toThrow(/only modify the module in the loader/);
          }
          const getInfo = Object.getOwnPropertyDescriptor(
            first,
            'buildInfo',
          )!.get!;
          expect(getInfo.call(second)).toBe(second.buildInfo);
          // A shared getter must resolve its receiver rather than the first instance.
          const getFiles = Object.getOwnPropertyDescriptor(
            first.buildInfo,
            buildInfoKeys[1],
          )!.get!;
          expect(getFiles.call(second.buildInfo)).toContain(second.resource);
          const assetsKey = buildInfoKeys[0];
          Object.defineProperty(first.buildInfo, assetsKey, {
            value: 'overridden',
          });
          expect(
            Object.getOwnPropertyDescriptor(second.buildInfo, assetsKey)!.get,
          ).toBeTypeOf('function');
        });
      });
    },
  ],
});
