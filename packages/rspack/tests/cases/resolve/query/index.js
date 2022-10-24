it("should make different modules for query", function () {
	var a = require("./empty");
	var b = require("./empty?1");
	var c = require("./empty?2");
	expect(typeof a).toBe("object");
	expect(typeof b).toBe("object");
	expect(typeof c).toBe("object");
	expect(a === b).toBe(false);
	expect(a === c).toBe(false);
	expect(b === c).toBe(false);
});
