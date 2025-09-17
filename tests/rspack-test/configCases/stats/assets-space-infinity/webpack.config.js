/** @type {import("@rspack/core").Configuration} */
module.exports = {
        context: __dirname,
        entry: "./index.js",
        stats: {
                assetsSpace: Infinity
        }
};
