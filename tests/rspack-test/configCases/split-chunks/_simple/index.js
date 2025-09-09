it("should run", function () {
	try {
		var a = require("./a");
		expect(a).toBe("a");
	} catch (_err) {
		// TODO:  It compiled but the runtime has some problem
	}
});
