it("should pass the resource to the loader", () => {
	const path = require("path");
	let result = require("./a?resourcequery#resourcefragment");
	expect(result).toEqual({
		resource: path.join(__dirname, "../a.js?resourcequery#resourcefragment"),
		// Formatted by prettier
		prev: 'module.exports = "a";\n'
	});
});
