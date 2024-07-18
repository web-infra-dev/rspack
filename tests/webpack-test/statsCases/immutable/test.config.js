const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			- asset fXXedXXbXXfeXXaXXbXX.js XX KiB [emitted] [immutable] (name: main)
			- asset XXecXXfbXXbddfXXec.js XX bytes [emitted] [immutable]
			+ asset XXefXXdebaXXcXX.js XX KiB [emitted] [immutable] (name: main)
			+ asset XXcXXaXXbXXdXXdcXX.js XX bytes [emitted] [immutable]"
		`);

	}
};
