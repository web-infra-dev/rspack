const {
  AngularWebpackPlugin,
  AngularWebpackLoaderPath
} = require("@ngtools/webpack");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const path = require("path");

const postCss = require("postcss");
const postCssLoaderPath = require.resolve("postcss-loader");

const componentsSourceMap = true;
const cssSourceMap = true;

const componentStyleLoaders = [
  {
    loader: require.resolve("css-loader"),
    options: {
      url: false,
      sourceMap: componentsSourceMap,
      importLoaders: 1,
      exportType: "string",
      esModule: false,
    },
  },
  //   {
  //     loader: postCssLoaderPath,
  //     options: {
  //       implementation: postCss,
  //       postcssOptions: postcssOptionsCreator(componentsSourceMap, false),
  //     },
  //   },
];

const globalStyleLoaders = [
  {
    loader: MiniCssExtractPlugin.loader,
  },
  {
    loader: require.resolve("css-loader"),
    options: {
      url: false,
      sourceMap: !!cssSourceMap,
      importLoaders: 1,
    },
  },
  //   {
  //     loader: postCssLoaderPath,
  //     options: {
  //       implementation: postCss,
  //       postcssOptions: postcssOptionsCreator(false, true),
  //       sourceMap: !!cssSourceMap,
  //     },
  //   },
];

/** @type {() => import('@rspack/cli').Configuration} */
module.exports = function () {
  const styleLanguages = [
    {
      extensions: ["css"],
      use: [],
    },
    {
      extensions: ["scss"],
      use: [
        {
          loader: require.resolve("resolve-url-loader"),
          options: {
            sourceMap: cssSourceMap,
          },
        },
        {
          loader: require.resolve("sass-loader"),
        },
      ],
    },
    {
      extensions: ["sass"],
      use: [
        {
          loader: require.resolve("resolve-url-loader"),
          options: {
            sourceMap: cssSourceMap,
          },
        },
        {
          loader: require.resolve("sass-loader"),
        },
      ],
    }
  ];
  /** @type {import('@rspack/cli').Configuration}  */
  let config = {
    target: "web",
    cache: false,
    mode: 'development',
    entry: "./src/main.ts",
    output: {
      path: path.resolve("./dist"),
    },
    devtool: "source-map",
    resolve: {
      extensions: [".ts", ".tsx", ".mjs", ".js"],
    },
    plugins: [
      new AngularWebpackPlugin({
        tsconfig: "./tsconfig.app.json",
      }),
    ],
    module: {
      rules: [
        {
          test: /\.?(svg|html)$/,
          // Only process HTML and SVG which are known Angular component resources.
          resourceQuery: /\?ngResource/,
          type: "asset/source",
        },
        {
          test: /\.ts$/,
          use: {
            loader: AngularWebpackLoaderPath,
          }
        },
        {
          test: /\.html$/,
          use: {
            loader: 'raw-loader'
          }
        },
        ...styleLanguages.map(({ extensions, use }) => ({
          test: new RegExp(`\\.(?:${extensions.join("|")})$`, "i"),
          rules: [
            // Setup processing rules for global and component styles
            {
              oneOf: [
                // Global styles are only defined global styles
                {
                  use: globalStyleLoaders,
                  resourceQuery: /\?ngGlobalStyle/,
                },
                // Component styles are all styles except defined global styles
                {
                  use: componentStyleLoaders,
                  resourceQuery: /\?ngResource/,
                },
              ],
            },
            { use },
          ],
        })),
      ],
    },
  };

  config

  return config;
};
