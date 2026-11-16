import type { Compiler } from '@rspack/core';

class MyStillValidModulePlugin {
  apply(compiler: Compiler) {
    const mockFn = rstest.fn();
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
    compiler.hooks.done.tap('MyStillValidModulePlugin', (stats) => {
      const cacheInfo = stats.toJson({ all: false, cacheInfo: true }).cacheInfo;
      const reused = cacheInfo?.['|Compilation/modules']?.reused ?? 0;
      expect(mockFn).toBeCalledTimes(reused);
    });
  }
}

export default MyStillValidModulePlugin;
