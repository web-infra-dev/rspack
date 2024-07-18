const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			- Entrypoint eXX XX KiB = runtime~eXX.js XX KiB eXX.js XX bytes
			- Entrypoint eXX XX KiB = runtime~eXX.js XX KiB eXX.js XX bytes
			- Rspack x.x.x compiled successfully
			+ Entrypoint eXX XX KiB = runtime~eXX.js XX KiB eXX.js XX KiB
			+ Entrypoint eXX XX KiB = runtime~eXX.js XX KiB eXX.js XX KiB
			+ webpack x.x.x compiled successfully"
		`);

	}
};
