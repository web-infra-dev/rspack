var webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
    mode: "production",
    entry: "./index",
    plugins: [
        new webpack.optimize.LimitChunkCountPlugin({
            maxChunks: 1
        })
    ],
    stats: {
        assets: true,
        chunkModules: true,
        dependentModules: true,
        chunkRelations: true,
        modules: false,
        chunks: true
    }
};
