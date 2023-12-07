it("should be able to use a context with a loader", () => {
	let a = "a";
	let result = require(`raw-loader!./${a}`).default;
	expect(result).toBe("export default 1;");
});
