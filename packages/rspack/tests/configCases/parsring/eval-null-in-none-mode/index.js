it("should evaluate null", function () {
	expect(null ? require("fail") : require("./a.js")).toBe("a");
	if (null) require("fail");
});
