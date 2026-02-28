class CustomPlugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap('applyPlugins', (compilation) => {
      compilation.hooks.processAssets.tapPromise(
        {
          name: 'applyPlugins',
        },
        async () => {
          compilation.emitAsset('apply-plugin.json', new compilation.compiler.rspack.sources.RawSource(JSON.stringify({
            secondary: true
          })))
        })
    })
  }
}
module.exports = CustomPlugin;