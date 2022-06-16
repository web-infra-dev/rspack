import Self from "../../../src";

module.exports = {
  entry: {
    entry1: { import: "./entryA.js", dependOn: "common" },
    common: "./entryB.js",
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [Self.loader, "css-loader"],
      },
    ],
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
