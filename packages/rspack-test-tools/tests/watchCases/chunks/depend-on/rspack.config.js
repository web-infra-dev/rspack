/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry: {
        main: "./main.js",
        shared: "./shared.js",
        index1: {
            import: "./index1.js",
            dependOn: "shared"
        },
        index2: {
            import: "./index2.js",
            dependOn: "shared"
        },
    },
    output: {
        filename: "[name].js"
    }
}
