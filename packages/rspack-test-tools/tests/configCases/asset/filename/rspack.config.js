const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    context: __dirname,
    module: {
        rules: [
            {
                test: /\.txt$/,
                type: "asset/resource",
                generator: {
                    filename({ filename }) {
                        return path.join("text", filename);
                    }
                }
            }
        ]
    }
};
