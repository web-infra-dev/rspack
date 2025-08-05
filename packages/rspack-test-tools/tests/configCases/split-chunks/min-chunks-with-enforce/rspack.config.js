/** @type {import("@rspack/core").Configuration} */
module.exports = {
    target: "node",
    entry: "./index.js",
    output: {
        filename: "[name].js"
    },
    optimization: {
        splitChunks: {
            minSize: 1,
            cacheGroups: {
                vendors: {
                    name: "vendors",
                    test: /[\\/]node_modules[\\/]/,
                    priority: 10,
                    minSize: 0,
                    maxSize: 0,
                    enforce: true,
                },
                vendorsCommon: {
                    test: /[\\/]node_modules[\\/]/,
                    name: "vendors-common",
                    minChunks: 2,
                    minSize: 0,
                    maxSize: 0,
                    priority: 12,
                    enforce: true,
                },
            }
        }
    },
    plugins: [
        {
            apply(compiler) {

            }
        }
    ]
};
