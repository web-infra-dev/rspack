import Self from "../../../src";

module.exports = {
  mode: "development",
  entry: ["./a.css", "./b.css"],
  output: {
    pathinfo: true,
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
