const { normalize, join } = require('path');

const PLUGIN_NAME = "Test";

class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
            compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
                const entry = Array.from(compilation.entries.values())[0];
                const entryDependency = entry.dependencies[0];
                const refModule = compilation.moduleGraph.getModule(entryDependency);
                expect(normalize(refModule.request)).toBe(normalize(join(__dirname, 'index.js')));
            });
        });

    }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    target: "web",
    node: {
        __dirname: false,
        __filename: false,
    },
    plugins: [
        new Plugin()
    ]
};
