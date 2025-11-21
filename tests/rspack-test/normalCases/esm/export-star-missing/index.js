it("should work well when export namespace with missing module", function () {
	let lib;
	try {
		lib = require("./lib")
	} catch (e) {
		expect(e.message).toBe("Cannot find module './missing'")
	}
	expect(lib).toBe(undefined)
});
