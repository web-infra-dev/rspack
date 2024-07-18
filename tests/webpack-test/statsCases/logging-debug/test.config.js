const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -4,1 +4,20 @@
			- Rspack x.x.x compiled <CLR=XX,BOLD>successfully</CLR> in X s
			+
			+ <CLR=XX,BOLD>DEBUG</CLR> <CLR=BOLD>LOG from ../logging/node_modules/custom-loader/index.js ../logging/node_modules/custom-loader/index.js!./index.js</CLR>
			+ <e> <CLR=XX,BOLD>An error</CLR>
			+ <w> <CLR=XX,BOLD>A warning</CLR>
			+ <-> <CLR=XX,BOLD>Unimportant</CLR>
			+ <i> <CLR=XX,BOLD>Info message</CLR>
			+ <CLR=BOLD>Just log</CLR>
			+ Just debug
			+ <t> <CLR=XX,BOLD>Measure: X ms</CLR>
			+ <-> <CLR=XX,BOLD>Nested</CLR>
			+ <CLR=BOLD>Log inside collapsed group</CLR>
			+ Trace
			+ <t> <CLR=XX,BOLD>Measure: X ms</CLR>
			+ -------
			+ <CLR=BOLD>After clear</CLR>
			+
			+ <CLR=XX,BOLD>DEBUG</CLR> <CLR=BOLD>LOG from ../logging/node_modules/custom-loader/index.js Named Logger ../logging/node_modules/custom-loader/index.js!./index.js</CLR>
			+ Message with named logger
			+
			+ webpack x.x.x compiled <CLR=XX,BOLD>successfully</CLR> in X ms"
		`);

	}
};
