// Cross a checkpoint inside an astral character, then exercise a long Unicode
// line and a large overlapping expression without checking in padded fixtures.
const source =
	`/*${"x".repeat(1021)}😀*/ require("./b");\r\n` +
	`/*${"é".repeat(900)}*/ require("./b");\n` +
	`module.exports = require("./b") && (\n/*${"padding\n".repeat(500)}*/ require("./b"));\n`;

module.exports = () => source;
module.exports.source = source;
