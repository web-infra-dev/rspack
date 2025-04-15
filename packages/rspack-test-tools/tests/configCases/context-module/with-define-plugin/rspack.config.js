const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    plugins: [
        new rspack.DefinePlugin({
            "process.env.DIR": JSON.stringify("sub"),
        }),
    ]
};
