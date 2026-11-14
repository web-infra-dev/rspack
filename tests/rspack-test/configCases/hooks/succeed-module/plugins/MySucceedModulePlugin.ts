import type { Compiler, Module } from '@rspack/core';

class MySucceedModulePlugin {
  apply(compiler: Compiler) {
    const mockFn = rstest.fn();
    compiler.hooks.compilation.tap('MySucceedModulePlugin', (compilation) => {
      const checkModule = (module: Module) => {
        if ('resource' in module && module.resource) {
          const source = module.originalSource();
          expect(source).not.toBeNull();
          const value = source!.source();
          expect(typeof value === 'string' || Buffer.isBuffer(value)).toBe(true);
          expect(source!.size()).toBe(
            Buffer.isBuffer(value) ? value.length : Buffer.byteLength(value, 'utf8'),
          );
          expect(source!.buffer().length).toBe(source!.size());
          const { source: sourceAndMapValue, map } = source!.sourceAndMap();
          expect(sourceAndMapValue).toBe(value);
          expect(source!.map()).toEqual(map);
        }
        mockFn();
      };
      compilation.hooks.succeedModule.tap('MySucceedModulePlugin', checkModule);
      compilation.hooks.stillValidModule.tap(
        'MySucceedModulePlugin',
        checkModule,
      );
    });
    compiler.hooks.done.tap('MySucceedModulePlugin', () => {
      expect(mockFn).toBeCalledTimes(4);
    });
  }
}

export default MySucceedModulePlugin;
