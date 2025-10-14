import value from "./a";

it("should run module.hot.accept(…)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(1);
	module?.hot?.accept("./a", function () {});
	NEXT(
		require("../../update")(done, true, () => {
			expect(value).toBe(2);
			done();
		})
	);
}));
