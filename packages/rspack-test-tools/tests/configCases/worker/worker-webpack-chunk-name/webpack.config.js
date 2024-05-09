/** @type {import("@rspack/core").Configuration} */
module.exports = {
  node: {
    __dirname: false,
  },
  optimization: {
    chunkIds: "named",
  }
}