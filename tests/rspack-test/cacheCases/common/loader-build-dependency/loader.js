const fs = require("fs");

module.exports = function () {
	const content = fs.readFileSync(__filename, "utf-8");
	const match = /LOADER_VALUE = (\d+)/.exec(content);
	return `export default ${match[1]};`;
};

// LOADER_VALUE = 1
---
const fs = require("fs");

module.exports = function () {
	const content = fs.readFileSync(__filename, "utf-8");
	const match = /LOADER_VALUE = (\d+)/.exec(content);
	return `export default ${match[1]};`;
};

// LOADER_VALUE = 2
