const path = require("path")
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh")
const rspack = require("@rspack/core")

const isProduction = process.env.NODE_ENV === "production"

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: isProduction ? "production" : "development",
  entry: {},
  context: __dirname,
  output: {
    // set uniqueName explicitly to make react-refresh works
    uniqueName: "lib2"
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        include: path.resolve(__dirname, "src"),
        use: {
          loader: "builtin:swc-loader",
          options: {
            jsc: {
							parser: {
								syntax: "ecmascript",
								jsx: true
							},
							transform: {
								react: {
									runtime: "automatic",
                  refresh: !isProduction,
								}
							}
						}
          }
        }
      }
    ]
  },
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: "mfeCCC",

      exposes: {
        "./Component": "./src/Component",
        "./Component2": "./src/LazyComponent"
      },

      shared: [
        // All (used) requests within lodash are shared.
        "lodash/",
        "date-fns",
        {
          react: {
            // Do not load our own version.
            // There must be a valid shared module available at runtime.
            // This improves build time as this module doesn't need to be compiled,
            // but it opts-out of possible fallbacks and runtime version upgrade.
            // import: false,
            import: false,
            singleton: true
          }
        }
      ]
    }),
    !isProduction && new ReactRefreshPlugin(),
  ],
  devServer: {
    port: 8082,
    // add CORS header for HMR chunk (xxx.hot-update.json and xxx.hot-update.js)
    headers: {
      'Access-Control-Allow-Origin': '*',
    }
  }
}