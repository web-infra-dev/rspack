var x = require("./module");

it("should allow to hot replace modules in a ConcatenatedModule", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(x).toEqual(nsObj({
		default: "ok1"
	}));
	module.hot.accept("./module", () => {
		x = require("./module");
		expect(x).toEqual(nsObj({
			default: "ok2"
		}));
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
