it("should pass the path to the loader", () => {
	const path = require("path");
	let result = require("./a");
	expect(result).toEqual({
		resourcePath: path.join(__dirname, "../a.js"),
		prev: 'module.exports = "a";'
	});
});
