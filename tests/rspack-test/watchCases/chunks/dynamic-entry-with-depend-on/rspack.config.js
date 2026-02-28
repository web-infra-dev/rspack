const fs = require("fs");
const path = require("path");

let MAIN;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry() {
        if (fs.existsSync(MAIN)) {
            return {
                shared: "./shared.js",
                main: {
                    import: "./main.js",
                    dependOn: "shared"
                },
            }
        }
        return {
            shared: "./shared.js"
        }
    },
    output: {
        filename: "[name].js"
    },
    plugins: [
        {
            apply(compiler) {
                MAIN = path.join(compiler.context, "main.js");

                compiler.hooks.finishMake.tap("PLUGIN", compilation => {
                    if (!fs.existsSync(MAIN)) {
                        compilation.missingDependencies.add(MAIN);
                    }
                });
            }
        }
    ]
}
