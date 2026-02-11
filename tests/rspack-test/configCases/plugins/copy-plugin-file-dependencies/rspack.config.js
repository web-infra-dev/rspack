const { CopyRspackPlugin } = require("@rspack/core");
const path = require("path");

module.exports = {
    entry: "./index.js",
    target: "node",
    plugins: [
        new CopyRspackPlugin({
            patterns: [
                {
                    from: path.join(__dirname, "public"),
                    to: path.join(__dirname, "dist"),
                }
            ]
        }),
        {
            apply(compiler) {
                compiler.hooks.done.tap("DonePlugin", (stats) => {
                    for (const file of stats.compilation.fileDependencies) {
                        expect(file).toBe(path.normalize(file));
                    }
                });
            }
        }
    ]
};
