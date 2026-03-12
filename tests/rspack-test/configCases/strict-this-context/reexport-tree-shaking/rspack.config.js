"use strict";

const rspack = require("@rspack/core");

/**
 * @param {boolean} strictThisContextOnImports 
 * @param {number} i
 * @returns {import("@rspack/core").Configuration}
 */
const config = (strictThisContextOnImports, i) => ({
    output: {
        filename: `bundle${i}.js`
    },
	module: {
		parser: {
			javascript: {
				strictThisContextOnImports
			}
		}
	},
    optimization: {
        concatenateModules: false,
    },
    plugins: [
        new rspack.DefinePlugin({
            "STRICT_THIS_CONTEXT_ON_IMPORTS": JSON.stringify(strictThisContextOnImports)
        })
    ]
});

module.exports = [true, false].map((strictThisContextOnImports, i) => config(strictThisContextOnImports, i));
