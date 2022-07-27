import Self from "../../../src";

module.exports = {
  entry: {
    entryA: "./entryA.js",
    entryB: "./entryB.js",
    entryC: "./entryC.js",
    entryD: "./entryD.js",
    entryE: "./entryE.js",
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [Self.loader, "css-loader"],
      },
    ],
  },
  output: {
    filename: "[name]-[contenthash].js",
  },
  plugins: [
    new Self({
      filename: "[contenthash].css",
    }),
  ],
};
