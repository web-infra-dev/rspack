const moduleValue = require("./module");
const external = require("external");
import referencer from "./referencer";

it("should keep the module hash when usage changes", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(moduleValue).toBe("module");
	expect(external).toBe("external");
	expect(referencer).toBe(42);
	module.hot.accept("./referencer", () => {
		expect(referencer).toBe("undefined undefined");
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
