it("should resolve with replaced request", function () {
	expect(require("./request.v1")).toBe("v2");
});
