var value = require("./file");

console.log("value should be 1, get ", value);

if (module.hot && module.hot.accept) {
	// `./file.js` is a dependency, which should be ended with `.js`
	// should fixed it.
	module.hot.accept("./file.js", () => {
		value = require("./file");
		console.log("value should be 2, get ", value);
	});
}

// it("should accept a dependencies and require a new value", done => {
// 	expect(value).toBe(1);
// 	module.hot.accept("./file", () => {
// 		value = require("./file");
// 		expect(value).toBe(2);
// 		outside();
// 		done();
// 	});
// 	NEXT(require("../../update")(done));
// });

// function outside() {
// 	expect(value).toBe(2);
// }
