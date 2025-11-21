const PLUGIN_NAME = "plugin";

class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
            compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
                expect(Array.from(compilation.entrypoints.keys())).toEqual([]);
            });

            compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
                expect(Array.from(compilation.entrypoints.keys())).toEqual(["main", "foo", "bar"]);
            })
        });
    }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
    entry: {
        main: {
            import: "./index.js",
        },
        foo: {
            import: "./foo.js",
            asyncChunks: true
        },
        bar: {
            import: "./bar.js",
            asyncChunks: true
        },
    },
    plugins: [new Plugin()]
};
