const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
    context: __dirname,
    entry: './index.js',
    module: {
        rules: [
            {
                test: /index.js/,
                use: [{ loader: "./access-mg-loader.js" }]
            }
        ]
    }
};
