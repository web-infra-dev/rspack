import vendor from "vendor";
import.meta.webpackHot.accept("vendor");
it("should hot update a splitted initial chunk", async () => {
	expect(vendor).toBe("1");
	await NEXT_HMR();
	expect(vendor).toBe("2");
});
