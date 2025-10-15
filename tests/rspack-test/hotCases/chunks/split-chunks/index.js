import vendor from "vendor";
module.hot.accept("vendor");
it("should hot update a splitted initial chunk", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(vendor).toBe("1");
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			expect(vendor).toBe("2");
			done();
		})
	);
}));
