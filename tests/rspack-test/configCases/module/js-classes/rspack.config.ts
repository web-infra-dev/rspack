import { defineConfig } from '@rspack/cli';
import {
  Module,
  NormalModule,
  ConcatenatedModule,
  ContextModule,
  ExternalModule,
} from '@rspack/core';

export default defineConfig({
  mode: 'production',
  target: 'node',
  optimization: { minimize: false, concatenateModules: true },
  resolve: { alias: { ignored: false } },
  externals: { 'external-value': 'commonjs node:path' },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('ModuleClasses', (compilation) => {
        const constructors = [
          Module,
          NormalModule,
          ConcatenatedModule,
          ContextModule,
          ExternalModule,
        ];
        for (const Constructor of constructors) {
          const unwrapped = new Constructor();
          expect(unwrapped).toBeInstanceOf(Constructor);
          const other = new Constructor();
          for (const key of Object.keys(unwrapped)) {
            const left = Object.getOwnPropertyDescriptor(unwrapped, key)!;
            const right = Object.getOwnPropertyDescriptor(other, key)!;
            expect(left.get).toBe(right.get);
            expect(left.set).toBe(right.set);
          }
          if (
            unwrapped instanceof NormalModule &&
            other instanceof NormalModule
          ) {
            Reflect.set(unwrapped, 'userRequest', 'first');
            Reflect.set(other, 'userRequest', 'second');
            expect(unwrapped.userRequest).toBe('first');
            expect(other.userRequest).toBe('second');
          }

          expect(() => unwrapped.size()).toThrow(/Invalid Module receiver/);
          if (Constructor !== Module) {
            expect(Object.getPrototypeOf(Constructor.prototype)).toBe(
              Module.prototype,
            );
            expect(Object.getPrototypeOf(Constructor)).toBe(Module);
            expect(Object.hasOwn(Constructor.prototype, 'size')).toBe(false);
            expect(Object.hasOwn(Constructor.prototype, 'originalSource')).toBe(
              false,
            );
          }
        }
        compilation.hooks.processAssets.tap('ModuleClasses', () => {
          const modules = [...compilation.modules];
          for (const Constructor of constructors) {
            expect(
              modules.some((module) => module.constructor === Constructor),
            ).toBe(true);
          }
          for (const module of modules) {
            expect(module).toBeInstanceOf(Module);
            expect(
              [...compilation.modules].find(
                (other) => other.identifier() === module.identifier(),
              ),
            ).toBe(module);
            expect(Module.prototype.size.call(module)).toBe(module.size());
            expect(module.readableIdentifier()).toBeTypeOf('string');
            expect(
              compilation.moduleGraph.getIssuer(module),
            ).not.toBeUndefined();
            expect(
              Object.hasOwn(module, Symbol.for('rspack.module.identifier')),
            ).toBe(true);
          }
          const normal = modules.find(
            (module): module is NormalModule => module instanceof NormalModule,
          )!;
          const concatenated = modules.find(
            (module): module is ConcatenatedModule =>
              module instanceof ConcatenatedModule,
          )!;
          expect(concatenated.rootModule).toBeInstanceOf(NormalModule);
          expect(concatenated.modules).toContain(concatenated.rootModule);
          expect(concatenated.rootModule).toBe(concatenated.rootModule);
          const matchResource = Object.getOwnPropertyDescriptor(
            normal,
            'matchResource',
          )!.get!;
          expect(() => matchResource.call(concatenated)).toThrow(
            /not a NormalModule/,
          );
          const setMatchResource = Object.getOwnPropertyDescriptor(
            normal,
            'matchResource',
          )!.set!;
          expect(() => setMatchResource.call(concatenated, undefined)).toThrow(
            /not a NormalModule/,
          );
          const root = Object.getOwnPropertyDescriptor(
            ConcatenatedModule.prototype,
            'rootModule',
          )!.get!;
          expect(() => root.call(normal)).toThrow(/not a ConcatenatedModule/);
          for (const receiver of [
            {},
            Object.create(NormalModule.prototype),
            new Proxy(normal, {}),
            compilation,
          ]) {
            expect(() => Module.prototype.size.call(receiver)).toThrow();
            expect(() => compilation.moduleGraph.getIssuer(receiver)).toThrow();
          }
          // Inherited methods should observe one patch on the base prototype.
          const size = Module.prototype.size;
          try {
            Module.prototype.size = () => 123;
            for (const module of modules) expect(module.size()).toBe(123);
          } finally {
            Module.prototype.size = size;
          }
        });
      });
    },
  ],
});
