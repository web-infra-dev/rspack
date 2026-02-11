const { CopyRspackPlugin } = require("@rspack/core");
const path = require("path");

module.exports = {
    entry: "./index.js",
    target: "node",
    plugins: [
        new CopyRspackPlugin({
            patterns: [
                {
                    from: "./public"
                }
            ]
        }),
        {
            apply(compiler) {
                compiler.hooks.done.tap("DonePlugin", (stats) => {
                    for (const file of stats.compilation.fileDependencies) {
                        // Verify that fileDependencies are always normalized
                        expect(file).toBe(path.normalize(file));
                    }
                });
            }
        }
    ]
};
