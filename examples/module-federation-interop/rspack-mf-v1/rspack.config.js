const path = require("path")
const HtmlWebpackPlugin = require("html-webpack-plugin")
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh")
const rspack = require("@rspack/core")

const isProduction = process.env.NODE_ENV === "production"
const containerName = "Rspack_MF_v1";

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: isProduction ? "production" : "development",
  entry: "./src/index.js",
  context: __dirname,
  devtool: false,
  output: {
    uniqueName: containerName,
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
    new HtmlWebpackPlugin({ excludeChunks: [containerName] }),
    new rspack.container.ModuleFederationPluginV1({
      name: containerName,
      remotes: {
        "Rspack_MF_v1_5": "Rspack_MF_v1_5@http://localhost:8081/Rspack_MF_v1_5.js",
        "Webpack_MF": "Webpack_MF@http://localhost:8082/Webpack_MF.js"
      },
      exposes: {
        "./Component": "./src/Component",
      },
      shared: {
        react: {
          singleton: true,
        }
      }
    }),
    !isProduction && new ReactRefreshPlugin(),
  ],
  devServer: {
    port: 8080,
    headers: {
      'Access-Control-Allow-Origin': '*',
    }
  }
}