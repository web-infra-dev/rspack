const config = require("./jest.config");

/** @type {import("jest").Config} */
module.exports = {
	...config,
	testMatch: ["<rootDir>/tests/*.hottest.js"]
};

module.exports.reporters.push([
	"jest-html-reporters",
	{
		filename: "hot.html",
		includeConsoleLog: true,
		inlineSource: true,
		pageTitle: "rspack tests: hot"
	}
]);
