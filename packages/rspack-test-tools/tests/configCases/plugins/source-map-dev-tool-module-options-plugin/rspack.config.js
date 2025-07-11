const { rspack } = require("@rspack/core");

/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
    node: {
        __dirname: false,
        __filename: false
    },
    output: {
        filename: "[name].js"
    },
    devtool: false,
    plugins: [
        new rspack.experiments.SourceMapDevToolModuleOptionsPlugin({
            module: true,
            cheap: true
        }),
        compiler => {
            compiler.hooks.finishMake.tap("PLUGIN", compilation => {
                for (const module of compilation.modules) {
                    expect(module.useSourceMap).toBe(true);
                    expect(module.useSimpleSourceMap).toBe(true);
                }
            });
        }
    ]
};
