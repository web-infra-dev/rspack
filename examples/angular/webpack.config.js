const rspackConfig = require("./rspack.config");
const TerserPlugin = require('terser-webpack-plugin')
delete rspackConfig.builtins;
const HtmlWebpackPlugin = require('html-webpack-plugin')

rspackConfig.optimization.concatenateModules = false;
rspackConfig.optimization.mangleExports = false;
// rspackConfig.optimization.innerGraph = false;
rspackConfig.plugins.push(new HtmlWebpackPlugin())
// rspackConfig.optimization.minimizer = [
// 	new TerserPlugin({
// 		minify: TerserPlugin.terserMinify,
// 		// `terserOptions` options will be passed to `swc` (`@swc/core`)
// 		// Link to options - https://swc.rs/docs/config-js-minify
// 		terserOptions: {}
// 	})
// ];
// rspackConfig.optimization.providedExports = false;
// rspackConfig.optimization.usedExports = false;
// rspackConfig.optimization.sideEffects = 'flag';
module.exports = rspackConfig;
