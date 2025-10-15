var value = require("./parent-file");

it("should bubble update from a nested dependency", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(1);
	module.hot.accept("./parent-file", () => {
		value = require("./parent-file");
		expect(value).toBe(2);
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
