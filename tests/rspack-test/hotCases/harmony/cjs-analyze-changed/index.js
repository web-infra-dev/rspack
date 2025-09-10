import value from "./reexport";

it("should generate code correctly when outgoing module changes its exports type", (done) => {
	expect(value.default).toBe(1);
	module.hot.accept("./reexport", () => {
		expect(value).toBe(1);
		done();
	});
	NEXT(require("../../update")(done));
});
