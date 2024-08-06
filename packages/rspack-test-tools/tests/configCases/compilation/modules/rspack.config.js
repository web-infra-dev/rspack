const rspack = require("@rspack/core");

const PLUGIN_NAME = "plugin";

class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.make.tap(PLUGIN_NAME, (compilation) => {
            compilation.hooks.processAssets.tap(
                {
                    name: PLUGIN_NAME,
                    stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
                },
                () => {
                    const module = Array.from(compilation.modules).find(module => module.rawRequest === "./index.js");
                    const block = module.blocks[0];
                    expect(block.dependencies[0].request).toBe("./a");
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
