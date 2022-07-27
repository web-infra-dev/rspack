import Self from "../../../src";

module.exports = {
  entry: {
    main: "./index.js",
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
    (() => {
      const self = new Self({ filename: "constructed.css" });

      self.options.filename = "mutated.css";

      return self;
    })(),
  ],
};
