it("should error importModule when a cycle with 2 modules is requested", () => {
	expect(require("./loader!./2/a")).toEqual([
		["./b.json", [["./a.json", [["./b.json", "err"]]]]]
	]);
});
it("should error importModule when a cycle with 3 modules is requested", () => {
	expect(require("./loader!./3/a")).toEqual([
		["./b.json", [["./c.json", [["./a.json", [["./b.json", "err"]]]]]]]
	]);
});
it("should error importModule when requesting itself", () => {
	expect(require("./loader!./1/a")).toEqual([
		["./a.json", [["./a.json", "err"]]]
	]);
});
it("should not report a cycle when importModule is used twice", () => {
	expect(require("./loader!./4/a")).toEqual([
		["./b.json", [["./c.json", []]]],
		["./b.json", [["./c.json", []]]]
	]);
});
