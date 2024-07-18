const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,1 +1,1 @@
			- asset react.js XX KiB [emitted] (name: react)
			+ asset react.js XX KiB [emitted] [minimized] (name: react) XX related asset
			@@ -3,3 +3,3 @@
			- ../../../../node_modules/.pnpm/react@XX.XX/node_modules/react/index.js XX bytes [built] [code generated]
			- ../../../../node_modules/.pnpm/react@XX.XX/node_modules/react/cjs/react.production.min.js XX KiB [built] [code generated]
			- Rspack x.x.x compiled successfully in X.XX
			+ ../../../node_modules/react/index.js XX bytes [built] [code generated]
			+ ../../../node_modules/react/cjs/react.production.min.js XX KiB [built] [code generated]
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
