it("should pass the target to the loader", () => {
	const result = require("./a");
	expect(result).toEqual({
		target: "web",
		prev: 'module.exports = "a";\n'
	});
});
