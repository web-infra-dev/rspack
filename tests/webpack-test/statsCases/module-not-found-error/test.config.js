const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,7 +1,2 @@
			- ERROR in ./index.js
			- × Module not found: Can't resolve 'buffer' in 'Xdir/module-not-found-error'
			- ╭─[XX:XX]
			- XX │ require('buffer')
			- · ─────────────────
			- XX │ require('os')
			- ╰────
			+ ERROR in ./index.js XX:XX-XX
			+ Module not found: Error: Can't resolve 'buffer' in 'Xdir/module-not-found-error'
			@@ -9,7 +4,2 @@
			- ERROR in ./index.js
			- × Module not found: Can't resolve 'os' in 'Xdir/module-not-found-error'
			- ╭─[XX:XX]
			- XX │ require('buffer')
			- XX │ require('os')
			- · ─────────────
			- ╰────
			+ BREAKING CHANGE: webpack < XX used to include polyfills for node.js core modules by default.
			+ This is no longer the case. Verify if you need this module and configure a polyfill for it.
			@@ -17,1 +7,19 @@
			- Rspack compiled with XX errors
			+ If you want to include a polyfill, you need to:
			+ - add a fallback 'resolve.fallback: { \\\\buffer\\\\: require.resolve(\\\\buffer/\\\\) }'
			+ - install 'buffer'
			+ If you don't want to include a polyfill, you can use an empty module like this:
			+ resolve.fallback: { \\\\buffer\\\\: false }
			+
			+ ERROR in ./index.js XX:XX-XX
			+ Module not found: Error: Can't resolve 'os' in 'Xdir/module-not-found-error'
			+
			+ BREAKING CHANGE: webpack < XX used to include polyfills for node.js core modules by default.
			+ This is no longer the case. Verify if you need this module and configure a polyfill for it.
			+
			+ If you want to include a polyfill, you need to:
			+ - add a fallback 'resolve.fallback: { \\\\os\\\\: require.resolve(\\\\os-browserify/browser\\\\) }'
			+ - install 'os-browserify'
			+ If you don't want to include a polyfill, you can use an empty module like this:
			+ resolve.fallback: { \\\\os\\\\: false }
			+
			+ webpack compiled with XX errors"
		`);

	}
};
