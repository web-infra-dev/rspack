import ok from "./module";

it("should abort when module is not accepted", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(ok).toBe("ok1-inner");
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			expect(ok).toBe("ok2");
			NEXT(
				require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
					expect(ok).toBe("ok3-inner");
					done();
				})
			);
		})
	);
}));

if (module.hot) {
	module.hot.accept("./module");
}
