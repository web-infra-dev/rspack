import fs from "fs";
import path from "path";
it("chunks/async-two_js.js should exist", function (done) {
	import("./two");
	expect(
		fs.statSync(path.join(__dirname, "chunks/async-two_js.js")).isFile()
	).toBe(true);
	// 	.then(function (two) {
	// 		expect(two.default).toEqual(2);
	// 		done();
	// 	})
	// 	.catch(function (err) {
	// 		done(err);
	// 	});
});
