import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class ExecutionInterceptorPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap(
      'ExecutionInterceptorPlugin',
      (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'ExecutionInterceptorPlugin',
          (_chunk, runtimeRequirements) => {
            runtimeRequirements.add(
              compiler.rspack.RuntimeGlobals.interceptModuleExecution,
            );
          },
        );
      },
    );
  }
}

export default defineConfig({
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new ExecutionInterceptorPlugin()],
});
