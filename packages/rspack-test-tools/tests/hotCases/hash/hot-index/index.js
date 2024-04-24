import value from './file'

it("should accept a dependencies and require a new value", (done) => {
	expect(value).toBe(1);
	NEXT(require("../../update")(done, true, () => {
		expect(value).toBe(2);
		NEXT(require("../../update")(done, true, () => {
			expect(value).toBe(1);
			NEXT(require("../../update")(done, true, () => {
				expect(value).toBe(3);
				done();
			}))
		}));
	}));
});

module.hot.accept("./file");