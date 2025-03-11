class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        const { ContextModule } = compiler.webpack;

        compiler.hooks.afterEmit.tap("PLUGIN", compilation => {
            const contextModule = Array.from(compilation.modules).find(module => module instanceof ContextModule);
            expect(contextModule.constructor.name).toBe("ContextModule");
        });
    }
}


/** @type {import("@rspack/core").Configuration} */
module.exports = {
    plugins: [new Plugin()]
};
