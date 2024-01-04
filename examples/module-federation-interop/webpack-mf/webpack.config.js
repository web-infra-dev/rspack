const path = require("path")
const HtmlWebpackPlugin = require("html-webpack-plugin")
const ReactRefreshPlugin = require("@pmmmwh/react-refresh-webpack-plugin")
const webpack = require("webpack")

const isProduction = process.env.NODE_ENV === "production"
const containerName = "Webpack_MF"

/** @type {import('webpack').Configuration} */
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
          loader: "swc-loader",
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
    new webpack.container.ModuleFederationPlugin({
      name: containerName,
      remotes: {
        "Rspack_MF_v1": "Rspack_MF_v1@http://localhost:8080/Rspack_MF_v1.js",
        "Rspack_MF_v1_5": "Rspack_MF_v1_5@http://localhost:8081/Rspack_MF_v1_5.js"
      },
      exposes: {
        "./Component": "./src/Component",
      },
      shared: {
        react: {
          singleton: true
        }
      }
    }),
    !isProduction && new ReactRefreshPlugin(),
  ],
  devServer: {
    port: 8082,
    headers: {
      'Access-Control-Allow-Origin': '*',
    }
  }
}