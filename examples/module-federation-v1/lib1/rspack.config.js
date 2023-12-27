const path = require("path")
const HtmlWebpackPlugin = require("html-webpack-plugin")
const rspack = require("@rspack/core")

const isProduction = process.env.NODE_ENV === "production"

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: isProduction ? "production" : "development",
  entry: "./src/index.js",
  context: __dirname,
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
    })
  ],
  devServer: {
    port: 8081,
  }
}
