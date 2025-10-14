var value = require("../file");
import('./async.js'); // make sure ensure chunk runtime added
it("should accept a dependencies multiple times", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(1);
	module.hot.accept("../file", () => {
		var oldValue = value;
		value = require("../file");
		expect(value).toBe(oldValue + 1);
		if (value < 4)
			NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
		else
			done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
