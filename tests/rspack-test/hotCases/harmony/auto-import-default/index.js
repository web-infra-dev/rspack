import value from "./file";

it("should auto-import an ES6 imported default value from non-ESM module on accept", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(1);
	module.hot.accept("./file", () => {
		expect(value).toBe(2);
		outside();
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));

function outside() {
	expect(value).toBe(2);
}
