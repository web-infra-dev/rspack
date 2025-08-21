const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
    context: __dirname,
    entry: {
        main: "./loader!"
    },
    stats: "none",
    mode: "development",
    plugins: [new rspack.HtmlRspackPlugin(), new rspack.CssExtractRspackPlugin()],
    experiments: {
        lazyCompilation: true
    },
    devServer: {
        hot: true
    }
};
