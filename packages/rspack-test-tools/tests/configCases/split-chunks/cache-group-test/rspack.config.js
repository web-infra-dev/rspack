/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry: "./index.js",
    target: "node",
    output: {
        filename: "[name].js"
    },
    optimization: {
        splitChunks: {
            cacheGroups: {
                common: {
                    test(module) {
                        expect(module.size()).toBe(5);
                        return true;
                    }
                }
            }
        }
    }

};
