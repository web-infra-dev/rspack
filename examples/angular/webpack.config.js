const webpack = require('webpack')
const rspackConfig = require("./rspack.config");
const TerserPlugin = require('terser-webpack-plugin')
delete rspackConfig.builtins;
const HtmlWebpackPlugin = require('html-webpack-plugin')

// Noticed that, rspack don't support scope hoisting and export mangling for now, they are known issues.
// So before you are trying to report code size inflation issue, please uncomment these two lines below at first and run again


rspackConfig.optimization.concatenateModules = false;
rspackConfig.optimization.mangleExports = false;
rspackConfig.plugins.push(new HtmlWebpackPlugin())
rspackConfig.plugins.push(
  new webpack.DefinePlugin({
    ngDevMode: false,
  })
)
module.exports = rspackConfig;
