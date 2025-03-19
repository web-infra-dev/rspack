const { rspack } = require("@rspack/core");

const PLUGIN_NAME = "plugin";

class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
            compilation.hooks.processAssets.tap(
                {
                    name: PLUGIN_NAME,
                    stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
                },
                () => {
                    compilation.emitAsset("/foo.txt", new compiler.webpack.sources.RawSource("foo"));
                }
            )
        });
    }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
    entry: "./index.js",
    plugins: [new Plugin()]
};
