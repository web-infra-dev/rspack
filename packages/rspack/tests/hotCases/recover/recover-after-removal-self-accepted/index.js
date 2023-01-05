import ok from "./module";

it("should abort when module is not accepted", done => {
	expect(ok).toBe("ok1-inner");
	NEXT(
		require("../../update")(done, true, () => {
			// should be ok
			expect(require('./module').default).toBe("ok2");
			NEXT(
				require("../../update")(done, true, () => {
					// should be ok
					expect(require('./module').default).toBe("ok3-inner");
					done();
				})
			);
		})
	);
});

if (module.hot) {
	module.hot.accept("./module");
}
