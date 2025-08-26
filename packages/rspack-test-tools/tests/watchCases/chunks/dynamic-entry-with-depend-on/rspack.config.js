const fs = require("fs");
const path = require("path");

let CONTEXT;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry() {
        if (fs.existsSync(path.join(CONTEXT, "main.js"))) {
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
                CONTEXT = compiler.context;
                compiler.hooks.finishMake.tap("PLUGIN", compilation => {
                    compilation.missingDependencies.add(path.join(CONTEXT, "main.js"));
                });
            }
        }
    ]
}
