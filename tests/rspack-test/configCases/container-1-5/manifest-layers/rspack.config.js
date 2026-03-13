const { ModuleFederationPlugin } = require("@rspack/core").container;

const commonConfig = {
  optimization: {
    minimize: false,
    chunkIds: "named",
    moduleIds: "named",
  },
  output: {
    ignoreBrowserWarnings: true,
    publicPath: "/",
  },
  target: "async-node",
};

module.exports = [
  {
    ...commonConfig,
    entry: "./provider/index.js",
    experiments: {
      layers: true,
    },
    module: {
      rules: [
        {
          test: /\.js$/,
          layer: "react-layer",
        },
        {
          test: /react\.js$/,
          issuerLayer: "react-layer",
          loader: require.resolve("./provider/layered-react-loader.js"),
        },
      ],
    },
    output: {
      ...commonConfig.output,
      filename: "provider/[name].js",
      chunkFilename: "provider/[id].js",
      uniqueName: "manifest-layers-provider",
    },
    plugins: [
      new ModuleFederationPlugin({
        name: "manifest_layers_provider",
        filename: "provider/container.js",
        library: {
          type: "commonjs-module",
          name: "manifest_layers_provider",
        },
        exposes: {
          "./ComponentA": "./provider/ComponentA.js",
        },
        shared: {
          react: {
            version: "0.1.2",
            singleton: true,
            requiredVersion: false,
            shareScope: "react-layer",
            layer: "react-layer",
            issuerLayer: "react-layer",
          },
        },
      }),
    ],
  },
  {
    ...commonConfig,
    entry: "./host/App.js",
    experiments: {
      layers: true,
    },
    module: {
      rules: [
        {
          test: /\.js$/,
          layer: "react-layer",
        },
      ],
    },
    output: {
      ...commonConfig.output,
      filename: "[name].js",
      chunkFilename: "[id].js",
      uniqueName: "manifest-layers-host",
    },
    plugins: [
      new ModuleFederationPlugin({
        name: "manifest_layers_host",
        filename: "container.js",
        library: { type: "commonjs-module" },
        manifest: true,
        exposes: {
          "./local-component": {
            import: "./host/ComponentA.js",
            layer: "react-layer",
          },
        },
        remotes: {
          containerA: {
            external: "./provider/container.js",
            shareScope: ["react-layer", "default"],
          },
        },
        shared: {
          react: {
            request: "react",
            import: false,
            shareKey: "react",
            singleton: true,
            requiredVersion: false,
            shareScope: "react-layer",
            layer: "react-layer",
            issuerLayer: "react-layer",
          },
        },
      }),
    ],
  },
];
