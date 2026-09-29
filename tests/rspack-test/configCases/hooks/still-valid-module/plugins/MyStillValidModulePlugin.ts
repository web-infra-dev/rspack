import type { Compiler } from '@rspack/core';

const mockFn = rstest.fn();

class MyStillValidModulePlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'MyStillValidModulePlugin',
      (compilation) => {
        compilation.hooks.stillValidModule.tap(
          'MyStillValidModulePlugin',
          () => {
            mockFn();
          },
        );
      },
    );
    compiler.hooks.done.tap('MyStillValidModulePlugin', () => {
      expect(mockFn).toBeCalledTimes(0);
    });
  }
}

export default MyStillValidModulePlugin;
