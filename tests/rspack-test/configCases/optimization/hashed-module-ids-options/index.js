it("should have unique hashed module ids with custom options", function () {
	var ids = [];
	for (var i = 1; i <= 5; i++) {
		var id = require("./files/file" + i + ".js");
		expect(typeof id).toBe("string");
		expect(id.length).toBeGreaterThanOrEqual(1);
		expect(ids.indexOf(id)).toBe(-1);
		ids.push(id);
	}
});
