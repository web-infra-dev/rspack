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
    new rspack.container.ModuleFederationPlugin({
      // List of remotes with URLs
      remotes: {
        "mfe-b": "mfeBBB@http://localhost:8081/mfeBBB.js",
        "mfe-c": "mfeCCC@http://localhost:8082/mfeCCC.js"
      },

      // list of shared modules with optional options
      shared: {
        // specifying a module request as shared module
        // will provide all used modules matching this name (version from package.json)
        // and consume shared modules in the version specified in dependencies from package.json
        // (or in dev/peer/optionalDependencies)
        // So it use the highest available version of this package matching the version requirement
        // from package.json, while providing it's own version to others.
        react: {
          singleton: true, // make sure only a single react module is used
        }
      },

      // list of runtime plugin modules (feature of MF1.5)
      runtimePlugins: [
        "./src/runtimePlugins/logger.js",
      ],
    })
  ],
  devServer: {
    port: 8080,
  }
}