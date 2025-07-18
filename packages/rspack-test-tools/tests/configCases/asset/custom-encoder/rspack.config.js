const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    mode: "development",
    plugins: [
        new rspack.CssExtractRspackPlugin(),
    ],
    module: {
        rules: [
            {
                test: /\.png$/,
                type: "asset/inline",
                generator: {
                    dataUrl() {
                        return "data:image/png;base64,custom-content";
                    }
                }
            },
            {
                test: /\.css/,
                use: [
                    rspack.CssExtractRspackPlugin.loader,
                    'css-loader',
                ],
            },
        ]
    },
    experiments: {
        css: false,
    }
};
