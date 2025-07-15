it("should be able to use a context with a loader", () => {
	const a = "a";
	const result = require(`./replace-loader!./${a}`).default;
	expect(result).toBe(2);
});
