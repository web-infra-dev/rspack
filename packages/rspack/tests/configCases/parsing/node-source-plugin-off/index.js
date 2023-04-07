it("should not load node bindings when node option is false", function () {
	console.log(typeof global);
	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8");
	expect(source.includes("console.log(typeof global)")).toBe(true);
	// expect((typeof global)).toBe("undefined");
});
