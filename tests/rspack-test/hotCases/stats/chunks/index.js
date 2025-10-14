import value from './file'

it("should correctly handle hot module replacement", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
    expect(value).toBe(1);
    NEXT(require("../../update")(done, true, () => {
        expect(value).toBe(2);
        done();
    }));
}));

module.hot.accept("./file");
