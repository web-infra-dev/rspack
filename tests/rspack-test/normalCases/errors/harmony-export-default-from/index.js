it("should throw if ESM export default from is used", () => {
	expect(() => require("./foo")).toThrowError(/Module parse failed/);
});
