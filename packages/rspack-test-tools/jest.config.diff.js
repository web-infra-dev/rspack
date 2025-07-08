const config = require("./jest.config");

/** @type {import('jest').Config} */
module.exports = {
	...config,
	testMatch: ["<rootDir>/tests/*.difftest.js"]
};

module.exports.reporters.push([
	"jest-html-reporters",
	{
		filename: "diff.html",
		includeConsoleLog: true,
		inlineSource: true,
		pageTitle: "rspack tests: diff"
	}
]);
