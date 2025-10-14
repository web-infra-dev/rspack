import a from "./a";

it("should abort when module is not accepted", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(a).toBe(1);
	NEXT(require("../../update")(done, {
		ignoreErrored: true
	}, () => {
		expect(a).toBe(1);
		NEXT(require("../../update")(done, {
			ignoreErrored: true
		}, () => {
			expect(a).toBe(3);
			done();
		}));
	}));
}));

if(module.hot) {
	module.hot.accept("./a");
}
