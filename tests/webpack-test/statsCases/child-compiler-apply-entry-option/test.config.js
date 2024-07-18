const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -5,2 +5,2 @@
			- asset child.js XX bytes [emitted] (name: child)
			- Entrypoint child XX bytes = child.js
			+ assets by status XX bytes [cached] XX asset
			+ Entrypoint child = child.js
			@@ -10,1 +10,7 @@
			- Rspack x.x.x compiled successfully in X.XX
			+
			+ WARNING in configuration
			+ The 'mode' option has not been set, webpack will fallback to 'production' for this value.
			+ Set 'mode' option to 'development' or 'production' to enable defaults for each environment.
			+ You can also set it to 'none' to disable any default behavior. Learn more: https://webpack.js.org/configuration/mode/
			+
			+ webpack x.x.x compiled with XX warning in X ms"
		`);

	}
};
