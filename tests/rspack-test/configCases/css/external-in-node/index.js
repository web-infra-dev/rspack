it("should import an external css", async () => {
	const x = await import("../external/style.css");
	expect(x).toEqual(nsObj({}));
});
