const PLUGIN_NAME = "plugin";

class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        const { Source } = compiler.webpack.sources;

        compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
            compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
                const entry = compilation.entries.get("main");
                const entryDependency = entry.dependencies[0];
                const entryModule = compilation.moduleGraph.getModule(entryDependency);
                const codeGenerationResult = compilation.codeGenerationResults.get(entryModule, "main");
                expect(codeGenerationResult.sources.get("javascript")).toBeInstanceOf(Source);
            });
        });
    }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
    entry: "./index.js",
    plugins: [new Plugin()]
};
