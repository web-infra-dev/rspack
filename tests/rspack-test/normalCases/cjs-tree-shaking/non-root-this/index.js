it("should not rewrite this nested in functions", () => {
	const f = require("./function.js").fff;
	expect(f.test1).toBe(true);
	expect(f.test2).toBe(true);
});
it("should not rewrite this nested in class", () => {
	const f = require("./class.js").fff;
	expect(f.test1).toBe(true);
	expect(f.test2).toBe(true);
});
it("should not rewrite this nested in arrow functions", () => {
	const f = require("./arrow.js").fff;
	expect(f.test1).toBe(true);
	expect(f.test2).toBe(true);
});
