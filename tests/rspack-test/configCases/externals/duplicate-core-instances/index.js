it("should support two JavaScript core instances sharing one native binding", function (done) {
	var fs = require("fs");
	expect(__webpack_modules__.external).toBeDefined();
	import("./chunk").then(ns => {
		expect(ns.readFile).toBe(fs.readFile);
		done();
	});
});
