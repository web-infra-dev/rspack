let moduleB;

class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.compilation.tap("Plugin", compilation => {
            compilation.hooks.processAssets.tap("Plugin", () => {
                if (moduleB) {
                    expect(() => {
                        moduleB.size();
                    }).toThrow(/Unable to access module with id =.* now. The module have been removed on the Rust side./)
                }
                moduleB = Array.from(compilation.modules).find(module => module.identifier().includes("b.js"));
            });
        });
    }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    plugins: [new Plugin()]
};
