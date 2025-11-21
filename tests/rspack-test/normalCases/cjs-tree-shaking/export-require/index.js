it("should import by export require with module.exports.abc", () => {
	const f = require("./module-exports.js");
	expect(f.abc).toBe("abc");
});
it("should import by export require with exports.abc", () => {
	const f = require("./exports.js");
	expect(f.abc).toBe("abc");
});
it("should import by export require with this.abc", () => {
	const f = require("./this.js");
	expect(f.abc).toBe("abc");
});
it("should import by export require with module.exports", () => {
	const f = require("./module-exports-whole.js");
	expect(f).toBe("abc");
});
