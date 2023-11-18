const path = require("path");
const rspack = require("@rspack/core");

const prod = process.env.NODE_ENV === "production";

/** @type {import('@rspack/cli').Configuration} */
const config = {
  context: __dirname,
  entry: "./src/main.tsx",
  target: ["web", "es5"],
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/,
        use: "sass-loader",
        type: "css",
      },
      {
        test: /\.module\.s[ac]ss$/,
        use: "sass-loader",
        type: "css/module",
      },
      {
        test: /\.(j|t)s$/,
        exclude: [/node_modules/],
        loader: "builtin:swc-loader",
        options: {
          sourceMap: true,
          jsc: {
            parser: {
              syntax: "typescript",
            },
            externalHelpers: true,
          },
        },
        type: "javascript/auto",
      },
      {
        test: /\.(t|j)sx$/,
        exclude: [/[\\/]node_modules[\\/]/],
        use: {
          loader: "builtin:swc-loader",
          options: {
            jsc: {
              parser: {
                syntax: "typescript",
                tsx: true,
              },
              transform: {
                react: {
                  runtime: "automatic",
                  development: !prod,
                },
              },
              externalHelpers: true,
            },
            rspackExperiments: {
              import: [
                {
                  libraryName: "@nutui/nutui-react",
                  customName: "@nutui/nutui-react/dist/esm/{{ member }}",
                  style: "{{ member }}/style/style.css",
                },
              ],
            },
          },
        },
        type: "javascript/auto",
      },
      {
        test: /\.(png|svg|jpg)$/,
        type: "asset/resource",
      },
    ],
  },
  resolve: {
    tsConfigPath: path.resolve(__dirname, "tsconfig.json"),
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  output: {
    publicPath: "/",
    filename: "[name].[contenthash].js",
  },
  optimization: {
    minimize: false,
    realContentHash: true,
    splitChunks: {
      cacheGroups: {
        someVendor: {
          chunks: "all",
          minChunks: 2,
        },
      },
    },
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      title: "NutUI React",
      template: path.join(__dirname, "index.html"),
    }),
    new rspack.ProgressPlugin({ prefix: "🐹 Rspack" }),
  ],
  experiments: {
    rspackFuture: {
      disableTransformByDefault: true,
    },
  },
};
module.exports = config;
