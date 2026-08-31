const {
  javascript: { JavascriptModulesPlugin },
  sources: { ConcatSource },
} = require('@rspack/core');

const PLUGIN = 'RenderContentTestPlugin';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  optimization: { minimize: false },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].chunk.js',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(PLUGIN, (compilation) => {
          const hooks =
            JavascriptModulesPlugin.getCompilationHooks(compilation);

          // A pass-through tap: returning the source unchanged must not disturb
          // the source the next tap receives.
          hooks.renderContent.tap(`${PLUGIN}::passthrough`, (source) => source);

          hooks.renderContent.tap(PLUGIN, (source, { chunk }) => {
            // The real consumer (webpack-target-webextension) compares the
            // render context chunk against the entrypoint chunk by identity.
            if (chunk.name === 'main') {
              const entryChunk = compilation.entrypoints
                .get('main')
                .getEntrypointChunk();
              expect(chunk).toBe(entryChunk);
            }
            return new ConcatSource(
              `globalThis.__rendered_${chunk.name}__ = true;\n`,
              `/* rendered:${chunk.name} */\n`,
              source,
              '\n/* end */',
            );
          });
        });
      },
    },
  ],
};
