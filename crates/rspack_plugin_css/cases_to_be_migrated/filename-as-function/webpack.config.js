import Self from "../../../src";

module.exports = {
  entry: {
    "demo/js/main": "./index.js",
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
    filename: "[name].js",
  },
  plugins: [
    new Self({
      filename: ({ chunk }) => `${chunk.name.replace("/js/", "/css/")}.css`,
    }),
  ],
};
