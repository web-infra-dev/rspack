const path = require("path");

it("node-polyfill", () => {
	expect(path.resolve("/a/b/c/../../d")).toBe("/a/d");
});
