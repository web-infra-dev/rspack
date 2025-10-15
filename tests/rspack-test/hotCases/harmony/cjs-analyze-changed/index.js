import value from "./reexport";

it("should generate code correctly when outgoing module changes its exports type", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value.default).toBe(1);
	module.hot.accept("./reexport", () => {
		expect(value).toBe(1);
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
