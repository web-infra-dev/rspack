import Self from "../../../src/index";

module.exports = {
  cache: { type: "filesystem" },
  entry: "bootstrap/dist/css/bootstrap.css",
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          Self.loader,
          {
            loader: "css-loader",
            options: {
              sourceMap: true,
            },
          },
        ],
      },
    ],
  },

  plugins: [
    new Self(),
    {
      apply(compiler) {
        compiler.hooks.infrastructureLog.tap("test", (origin, type, args) => {
          if (type === "warn" || type === "error") {
            throw new Error(`<${type}> [${origin}] ${args.toString()}`);
          }
        });
      },
    },
  ],
};
