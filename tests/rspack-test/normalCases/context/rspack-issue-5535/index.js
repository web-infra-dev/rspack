it("should be able to generate correct alternative requests", () => {
	const requireFiles = require("./test-module");
	expect(requireFiles("./test.module")).toBe(1);
	expect(requireFiles("./test.module.js")).toBe(1);
});
