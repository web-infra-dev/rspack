it("should throw if harmony export default from is used", () => {
	expect(() => require("./foo")).toThrowError(/Module parse failed/);
});
