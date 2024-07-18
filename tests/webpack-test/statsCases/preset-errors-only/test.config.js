const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(
			`"Compared values have no visual difference."`
		);

	}
};
