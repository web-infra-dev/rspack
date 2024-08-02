const PLUGIN_NAME = "plugin";

class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.afterCompile.tap(PLUGIN_NAME, compilation => {
            expect(Array.from(compilation.entries.keys())).toEqual(["main", "foo"]);

            const entry = compilation.entries.get("foo");
            expect(entry.dependencies.length).toEqual(1);
            expect(entry.options.asyncChunks).toEqual(true);

            compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
                compilation.entries.delete("foo");
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
        }
    },
    plugins: [new Plugin()]
};
