/** @type {import("@rspack/core").Configuration} */
module.exports = {
    mode: "development",
    module: {
        unsafeCache: true
    },
    plugins: [
        compiler => {
            compiler.hooks.done.tap("PLUGIN", stats => {
                const fileDependencies = Array.from(stats.compilation.fileDependencies)

                // With unsafeCache disabled, expect package.json and other node_modules 
                // dependency files to be included in fileDependencies
                const packageJsonDependencies = fileDependencies.filter(dependency => dependency.includes('node_modules/foo/package.json'));
                expect(packageJsonDependencies.length).toBe(0)

                // Module files themselves are still tracked (added in module.build())
                // This ensures user modifications to node_modules are detected
                const fooModuleFile = fileDependencies.find(dependency => dependency.endsWith('node_modules/foo/index.js'))
                expect(fooModuleFile).toBeTruthy()
            });
        }
    ]
};
