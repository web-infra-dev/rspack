it("should error loadModule when a cycle with 2 modules is requested", () => {
	expect(require("./loader.mjs!./2/a")).toMatch(
		/^source: err: There is a circular build dependency/
	);
});
it("should error loadModule when a cycle with 3 modules is requested", () => {
	expect(require("./loader.mjs!./3/a")).toMatch(
		/^source: source: err: There is a circular build dependency/
	);
});
it("should error loadModule when requesting itself", () => {
	expect(require("./loader.mjs!./1/a")).toMatch(
		/^err: There is a circular build dependency/
	);
});
