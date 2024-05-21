/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    function(compiler) {
      compiler.hooks.contextModuleFactory.tap(
        "test",
        contextModuleFactory => {
          contextModuleFactory.hooks.afterResolve.tap("test", resolveData => {
            console.log(resolveData)
            // return resolveData
          });
        }
      );
    }
  ]
}
