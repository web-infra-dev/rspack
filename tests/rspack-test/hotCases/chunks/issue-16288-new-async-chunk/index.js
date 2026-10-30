if (!globalThis.__RSPACK_16288_TEST_REGISTERED__) {
	globalThis.__RSPACK_16288_TEST_REGISTERED__ = true;
	it("should load a new split async dependency during HMR", async () => {
		if (!globalThis.__RSPACK_16288_TEST_STARTED__) {
			globalThis.__RSPACK_16288_TEST_STARTED__ = true;
			await import("./lazy");
			expect(globalThis.__RSPACK_16288__).toBe("v0");

			await NEXT_HMR();
		}

		expect(globalThis.__RSPACK_16288__).toBe("v1:function");
	});
}
import.meta.webpackHot.accept();
