class ExecutionInterceptorPlugin {
  apply(compiler) {
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

export default {
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new ExecutionInterceptorPlugin()],
};
