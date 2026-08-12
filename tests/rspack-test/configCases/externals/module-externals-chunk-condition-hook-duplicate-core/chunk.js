it("should share ExternalModule hooks when JavaScript core is loaded twice", function (done) {
	var fs = require("fs");
	expect(__webpack_modules__.external).toBeDefined();
	import("./chunk").then(ns => {
		expect(ns.readFile).toBe(fs.readFile);
		done();
	});
});
