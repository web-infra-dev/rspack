import type { Compiler } from '@rspack/core';

const mockFn = rstest.fn();

class MySucceedModulePlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('MySucceedModulePlugin', (compilation) => {
      compilation.hooks.succeedModule.tap('MySucceedModulePlugin', (module) => {
        if ('resource' in module && module.resource) {
          const source = module.originalSource();
          expect(source).not.toBeNull();
        }
        mockFn();
      });
    });
    compiler.hooks.done.tap('MySucceedModulePlugin', () => {
      expect(mockFn).toBeCalledTimes(4);
    });
  }
}

export default MySucceedModulePlugin;
