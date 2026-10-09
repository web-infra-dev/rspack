it("should resolve an ordered consume that is only reachable through an async chunk", async () => {
	expect(await import("lib")).toEqual(expect.objectContaining({ default: "lib" }));
});
