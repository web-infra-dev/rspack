/** @type {import("@rspack/core").Configuration} */
module.exports = {
    resolve: {
        alias: {
            "multi-alias": [
                "./b",
                "./a"
            ]
        }
    }
};
