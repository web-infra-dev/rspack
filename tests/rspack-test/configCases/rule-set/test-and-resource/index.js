it("should require both test and resource to match", function () {
	expect(require("./entry")).toEqual(["entry", "matched"]);
	expect(require("./runtime")).toEqual(["runtime"]);
	expect(require("./async-entry")).toEqual(["async-entry", "matched"]);
	expect(require("./async-runtime")).toEqual(["async-runtime"]);
});
