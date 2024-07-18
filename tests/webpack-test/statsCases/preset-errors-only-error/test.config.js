const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -6,6 +6,2 @@
			- ERROR in ./index.js
			- × Module not found: Can't resolve 'does-not-exist' in 'Xdir/preset-errors-only-error'
			- ╭────
			- XX │ require('does-not-exist')
			- · ─────────────────────────
			- ╰────
			+ ERROR in ./index.js XX:XX-XX
			+ Module not found: Error: Can't resolve 'does-not-exist' in 'Xdir/preset-errors-only-error'
			@@ -13,1 +9,1 @@
			- Rspack compiled with XX error
			+ webpack compiled with XX error"
		`);

	}
};
