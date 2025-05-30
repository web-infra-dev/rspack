it("should error importModule when a cycle with 2 modules is requested", () => {
	expect(require("./loader!./2/a")).toMatch("circular build dependency");
});
it("should error importModule when a cycle with 3 modules is requested", () => {
	expect(require("./loader!./3/a")).toMatch("circular build dependency");
});
it("should error importModule when requesting itself", () => {
	expect(require("./loader!./1/a")).toMatch("circular build dependency");
});
