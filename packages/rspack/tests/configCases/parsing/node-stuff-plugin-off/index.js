it("should not evaluate __dirname or __filename when node option is false", function (done) {
	console.log(__dirname, __filename);
	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8");
	expect(source.includes("console.log(__dirname, __filename)")).toBe(true);
	// if (typeof __dirname !== "undefined") {
	// 	done.fail();
	// }
	// if (typeof __filename !== "undefined") {
	// 	done.fail();
	// }
	done();
});
