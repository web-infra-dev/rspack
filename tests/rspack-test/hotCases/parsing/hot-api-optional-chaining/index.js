import value from "./a";

it("should run module.hot.accept(…)", function (done) {
	expect(value).toBe(1);
	module?.hot?.accept("./a", function () { });
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			expect(value).toBe(2);
			done();
		})
	);
});
