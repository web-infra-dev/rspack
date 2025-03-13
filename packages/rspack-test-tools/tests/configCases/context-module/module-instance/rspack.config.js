class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        const { ContextModule } = compiler.webpack;

        compiler.hooks.afterEmit.tap("PLUGIN", compilation => {
            const contextModule = Array.from(compilation.modules).find(module => module instanceof ContextModule);
            expect(contextModule.constructor.name).toBe("ContextModule");
            expect(contextModule.type).toBe("javascript/auto");
            expect("context" in contextModule).toBe(true);
            expect("layer" in contextModule).toBe(true);
            expect("factoryMeta" in contextModule).toBe(true);
            expect(contextModule.useSourceMap).toBe(false);
            expect(contextModule.useSimpleSourceMap).toBe(false);
            expect("buildMeta" in contextModule).toBe(true);
            expect("buildMeta" in contextModule).toBe(true);
        });
    }
}


/** @type {import("@rspack/core").Configuration} */
module.exports = {
    plugins: [new Plugin()]
};
