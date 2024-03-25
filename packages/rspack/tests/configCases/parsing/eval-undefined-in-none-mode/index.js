it("should evaluate undefined", function () {
	expect(undefined ? require("fail") : require("./a")).toBe("a");
	if (undefined) require("fail");
	undefined && require("fail");
	typeof undefined === "undefined" ? require("./a") : require("fail");
	if (typeof undefined !== "undefined") require("fail");
});
