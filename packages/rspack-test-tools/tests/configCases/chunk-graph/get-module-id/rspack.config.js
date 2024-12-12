class Plugin {
    apply(compiler) {
        compiler.hooks.compilation.tap("Test", compilation => {
            compilation.hooks.processAssets.tap("Test", () => {
                const module = Array.from(compilation.modules)[0];
                const moduleId = compilation.chunkGraph.getModuleId(module);
                expect(moduleId).toBeTruthy();
            });
        });
    }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    target: 'web',
    node: false,
    entry: {
        main: "./index.js"
    },
    output: {
        filename: "[name].js"
    },
    optimization: {
        sideEffects: false,
    },
    plugins: [
        new Plugin()
    ]
};
