it("should not include foo.js", async () => {
	await expect(
		eval("require")("node:fs/promises").readFile(__filename, "utf8")
	).resolves.not.toContain(["SHOULD_NOT", "BE_INCLUDED"].join("_"));
});
it("should throw as `foo.js` shouldn't be included", () => {
	expect(() => require("./not-parsed-a")).toThrow();
});
