import value from './file';

it("should store and resume asset parser and generator states", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe('string');
	module.hot.accept("./file", () => {
		expect(value).toBe('string result');
		done();
	});
	NEXT(require("../../update")(done));
}));
