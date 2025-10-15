import a from "./a";

it("should abort when module is declined by parent", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(a).toBe(1);
	NEXT(require("@rspack/test-tools/helper/legacy/update")((err) => {
		try {
			expect(err.message).toMatch(/Aborted because of declined dependency: \.\/b\.js in \.\/a\.js/);
			expect(err.message).toMatch(/Update propagation: \.\/c\.js -> \.\/b\.js -> \.\/a\.js/);
			done();
		} catch (e) {
			done(e);
		}
	}));
}));
