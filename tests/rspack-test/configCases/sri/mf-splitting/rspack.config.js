const rspack = require("@rspack/core");
const { SubresourceIntegrityPlugin } = rspack;

const createConfig = (runtimeChunk, mfPlugin) => ({
  mode: "production",
  devtool: false,
  target: "web",
  entry: {
    main: "./main-entry",
    mfeEntry: "./mfe-entry",
  },
  plugins: [
    new SubresourceIntegrityPlugin(),
    mfPlugin && new rspack.container.ModuleFederationPlugin({
      name: "mfeEntry",
      filename: 'mfe-chunk.js',
      exposes: {
        './mock': 'mock',
        './mock-redux': 'mock-redux',
      }
    }),
  ].filter(Boolean),
  output: {
    filename: `${runtimeChunk}-${mfPlugin ? "mf" : "no-mf"}-[name].js`,
    crossOriginLoading: "anonymous",
  },
  optimization: {
    runtimeChunk: runtimeChunk,
    splitChunks: {
      cacheGroups: {
        framework: {
          test(m) {
            const resource = m.nameForCondition?.();
            if (!resource) return false;

            const frameworkDeps = [
              "mock",
              "mock-redux",
            ];

            const isFramework = frameworkDeps
              .map((pkg) => `node_modules/${pkg}/`)
              .some((dep) => resource.includes(dep));

            return isFramework;
          },
          name: "framework",
          chunks: "all",
          enforce: true,
        },
      },
    },
  },
});


module.exports = [
  createConfig("single", true),
  createConfig("single", false),
  createConfig("multiple", true),
  createConfig("multiple", false),
];