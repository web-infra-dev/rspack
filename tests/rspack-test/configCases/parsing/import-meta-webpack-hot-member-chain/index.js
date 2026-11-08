it("should parse nested and computed import.meta.webpackHot member chains", () => {
	expect(import.meta.webpackHot.data?.components?.Counter).toBeUndefined();
	expect(import.meta.webpackHot["data"]).toBeUndefined();
	expect(import.meta["webpackHot"].data).toBeUndefined();
});
