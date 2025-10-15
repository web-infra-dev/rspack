import { value } from "./file";
import value2 from "./commonjs";

it("should auto-import multiple ES6 imported values on accept", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(1);
	expect(value2).toBe(10);
	module.hot.accept(["./file", "./commonjs"], () => {
		expect(value).toBe(2);
		expect(value2).toBe(20);
		outside();
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));

function outside() {
	expect(value).toBe(2);
	expect(value2).toBe(20);
}
