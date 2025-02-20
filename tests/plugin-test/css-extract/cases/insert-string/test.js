const fs = require("fs");
const path = require("path");
module.exports = function (outputDirectory, _stats) {
	const mainContent = fs.readFileSync(path.resolve(outputDirectory, "main.js"), "utf8");
	expect(mainContent).toContain(`var target = document.querySelector("script[src='1.js']")`);
};
