class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.compilation.tap("Plugin", compilation => {
            compilation.hooks.processAssets.tap("Plugin", () => {
                const module = Array.from(compilation.modules).find(module => module.buildInfo.affected);
                expect(!!module).toBe(true);
            });
        });
    }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    plugins: [new Plugin()]
};
