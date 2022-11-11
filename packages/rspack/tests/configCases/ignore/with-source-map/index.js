const fs = require("fs");

it("should ignore ignored resources", function () {
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	expect(source.length).not.toBe(0);
	expect(require("./ignored-module")).toEqual({});
});
