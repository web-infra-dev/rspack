import {val} from "./module";

it("should fail accept changes", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(val).toBe(1);
	NEXT(require("../../update")((err) => {
		try {
			expect(err.message).toMatch(/Aborted because \.\/node_modules\/dep1\/file.js is not accepted/);
			expect(err.message).toMatch(/Update propagation: \.\/node_modules\/dep1\/file.js -> \.\/node_modules\/dep1\/exports\.js -> \.\/module\.js -> \.\/index.js/);
			done();
		} catch(e) {
			done(e);
		}
	}));
}));
