/** @type {import("@rspack/core").Configuration} */
module.exports = {
    output: {
        // TODO https://github.com/webpack/webpack/issues/16599
        chunkFilename: "[id].[hash].js",
    },
}