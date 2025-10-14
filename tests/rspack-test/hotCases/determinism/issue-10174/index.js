import { c } from "./deps/a";
import hot from "./hot";

it("should only register changes for the changed module", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(hot).toBe(1);
	expect(c()).toBe(42);
	module.hot.accept("./hot", () => {
		expect(hot).toBe(2);
		expect(c()).toBe(42);
		done();
	});

	NEXT(require("../../update")(done));
}));
