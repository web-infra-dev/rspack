/*
Ensures proper cleanup of dependency subtree when 404 entry is removed.
Without proper cleanup, this would throw a "Module not found" error for './404.js'.

× Module not found: Can't resolve './404.js'
.         ╭────
.       1 │ require("./404.js")
.         · ───────────────────
.         ╰────
*/

const fs = require("fs");
const path = require("path");

let MAIN;
let _404;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry() {
        const entries = {};
        if (fs.existsSync(MAIN)) {
            entries["main"] = "./main.js";
        }
        if (fs.existsSync(_404)) {
            entries["404"] = "./404-page-loader.js!";
        }
        return entries;
    },
    output: {
        filename: "[name].js"
    },
    plugins: [
        {
            apply(compiler) {
                MAIN = path.join(compiler.context, "main.js");
                _404 = path.join(compiler.context, "404.js");

                compiler.hooks.finishMake.tap("PLUGIN", compilation => {
                    if (!fs.existsSync(MAIN)) {
                        compilation.missingDependencies.add(MAIN);
                    }
                    if (!fs.existsSync(_404)) {
                        compilation.missingDependencies.add(_404);
                    }
                });
            }
        }
    ]
}
