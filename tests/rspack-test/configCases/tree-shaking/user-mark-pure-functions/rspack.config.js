
module.exports = {
  mode: 'production',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false
  },
  experiments: {
    noSideEffectsNotation: true,
  },
  module: {
    rules: [
      {
        test: /decl\.js/,
        parser: {
          pureFunctions: ['pureFn', 'notExistFunction']
        }
      }
    ]
  },
  // DEBUG
  // plugins: [
  //   compiler => {
  //     compiler.hooks.done.tap('NoSideEffectsNotation', (stats) => {
  //       console.log(JSON.stringify(stats.toJson({
  //         optimizationBailout: true,
  //         all: false,
  //         modules: true,
  //       }), null, 2))
  //     })
  //   }
  // ]
}