const path = require("path")
const HtmlWebpackPlugin = require("html-webpack-plugin")
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh")
const rspack = require("@rspack/core")

const isProduction = process.env.NODE_ENV === "production"

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: isProduction ? "production" : "development",
  entry: "./src/index.js",
  context: __dirname,
  output: {
    // set uniqueName explicitly to make react-refresh works
    uniqueName: "lib1"
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
    new HtmlWebpackPlugin(),
    new rspack.container.ModuleFederationPluginV1({
      // A unique name
      name: "mfeBBB",
      // List of exposed modules
      exposes: {
        "./Component": "./src/Component",
      },

      // list of shared modules
      shared: [
        // date-fns is shared with the other remote, app doesn't know about that
        "date-fns",
        {
          react: {
            singleton: true // must be specified in each config
          }
        }
      ]
    }),
    !isProduction && new ReactRefreshPlugin(),
  ],
  devServer: {
    port: 8081,
    // add CORS header for HMR chunk (xxx.hot-update.json and xxx.hot-update.js)
    headers: {
      'Access-Control-Allow-Origin': '*',
    }
  }
}
