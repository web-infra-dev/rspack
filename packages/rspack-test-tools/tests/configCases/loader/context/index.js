it("should be able to use a context with a loader", () => {
	let a = "a";
	let result = require(`./replace-loader!./${a}`).default;
	expect(result).toBe(2);
});
