const assert = require("assert");

module.exports = function (content) {
	assert.ok(
		!content.includes("string"),
		"TypeScript should be already transformed into JavaScript"
	);
	content += "\nexport const lib2: string = 'lib2';";
	return content;
};
