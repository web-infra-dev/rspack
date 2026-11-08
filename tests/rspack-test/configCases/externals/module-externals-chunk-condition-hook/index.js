it("should run ExternalModule chunkCondition hooks from JavaScript", function (done) {
	var fs = require("fs");
	expect(__webpack_modules__.external).toBeDefined();
	import("./chunk").then(ns => {
		expect(ns.readFile).toBe(fs.readFile);
		done();
	});
});
