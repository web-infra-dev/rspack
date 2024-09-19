const { ContextReplacementPlugin } = require("@rspack/core");

module.exports = /** @type {import("@rspack/core").Configuration} */ ({
	plugins: [
		new ContextReplacementPlugin(/context-replacement.b$/, /^\.\/only/),
	]
});
