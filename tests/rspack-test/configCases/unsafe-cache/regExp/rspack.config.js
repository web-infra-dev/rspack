/** @type {import("@rspack/core").Configuration} */
module.exports = {
    mode: "development",
    module: {
        unsafeCache: /a\.js/
    },
    resolve: {
        extensions: [".json", ".js"]
    },
    plugins: [
        compiler => {
            compiler.hooks.done.tap("PLUGIN", stats => {
                const missingDependencies = Array.from(stats.compilation.missingDependencies)

                const aJsonDependencies = missingDependencies.filter(dependency => dependency.includes('a.json'));
                expect(aJsonDependencies.length).toBe(0)

                const bJsonDependencies = missingDependencies.filter(dependency => dependency.includes('b.json'));
                expect(bJsonDependencies.length).toBe(1)
            });
        }
    ]
};
