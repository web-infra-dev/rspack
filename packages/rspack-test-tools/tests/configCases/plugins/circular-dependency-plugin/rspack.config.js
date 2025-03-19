const { CircularDependencyRspackPlugin } = require("@rspack/core");

module.exports = {
    entry: {
        'aa': './require-circular/d.js',
        'bb': './import-circular/index.js',
        'cc': './no-cycle/index.js'
    },
    target: "node",
    plugins: [
        new CircularDependencyRspackPlugin({
            failOnError: false,
        })
    ]
};
