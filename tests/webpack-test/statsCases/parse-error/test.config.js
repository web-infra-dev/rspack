const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,4 +1,3 @@
			- asset main.js XX KiB [emitted] (name: main)
			- orphan modules XX bytes [orphan] XX modules
			- cacheable modules XX bytes
			- ./index.js + XX modules XX bytes [code generated]
			+ assets by status XX KiB [cached] XX asset
			+ orphan modules XX bytes [orphan] XX module
			+ ./index.js + XX modules XX bytes [built] [code generated]
			@@ -7,11 +6,10 @@
			- ERROR in ./b.js
			- × Module parse failed:
			- ╰─▶   × JavaScript parsing error: Expected ';', '}' or <eof>
			- ╭─[XX:XX]
			- XX │ includes
			- XX │ a
			- XX │ parser )
			- ·        ─
			- XX │ error
			- XX │ in
			- ╰────
			+ ERROR in ./b.js XX:XX
			+ Module parse failed: Unexpected token (XX:XX)
			+ You may need an appropriate loader to handle this file type, currently no loaders are configured to process this file. See https://webpack.js.org/concepts#loaders
			+ | includes
			+ | a
			+ > parser )
			+ | error
			+ | in
			+ @ ./a.js XX:XX-XX
			+ @ ./index.js XX:XX-XX
			@@ -19,6 +17,1 @@
			- help:
			- You may need an appropriate loader to handle this file type.
			- @ ./a.js
			- @ ./index.js
			-
			- Rspack x.x.x compiled with XX error
			+ webpack x.x.x compiled with XX error"
		`);

	}
};
