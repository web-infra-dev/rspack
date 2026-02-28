it("should pass the target to the loader", () => {
	let result = require("./a");
	expect(result).toEqual({
		target: "web",
		prev: 'module.exports = "a";\n'
	});
});
