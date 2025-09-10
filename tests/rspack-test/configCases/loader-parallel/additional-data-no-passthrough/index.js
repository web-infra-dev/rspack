it("should not passthrough additional data if builtin loader didn't reuse additional data", () => {
	let result = require("./a");
	expect(Object.keys(result)).not.toContain("a");
	expect(Object.keys(result)).toContain("b");
});
