const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,1 +1,1 @@
			- asset main.js XX KiB [emitted] (name: main)
			+ asset main.js XX bytes [emitted] (name: main)
			@@ -3,2 +3,11 @@
			- ./index.js + XX modules XX bytes [code generated]
			- Rspack x.x.x compiled successfully in X.XX
			+ ./index.js + XX modules XX bytes [built] [code generated]
			+
			+ WARNING in ./module.js?XX XX:XX-XX
			+ Should not import the named export 'homepage' (imported as 'homepage') from default-exporting module (only default export is available soon)
			+ @ ./index.js XX:XX-XX
			+
			+ WARNING in ./moduleXX.js?XX XX:XX-XX
			+ Should not import the named export 'name' (imported as 'name') from default-exporting module (only default export is available soon)
			+ @ ./index.js XX:XX-XX
			+
			+ webpack x.x.x compiled with XX warnings in X ms"
		`);

	}
};
