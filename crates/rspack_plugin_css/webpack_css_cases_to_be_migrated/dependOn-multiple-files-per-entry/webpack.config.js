import Self from "../../../src";

module.exports = {
  entry: {
    entry1: { import: ["./entryA.js", "./entryB.js"], dependOn: "common" },
    common: ["./entryC.js", "./entryD.js"],
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
