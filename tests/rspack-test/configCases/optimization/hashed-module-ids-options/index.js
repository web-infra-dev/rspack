it("should respect custom hashDigest and hashDigestLength options", function () {
	var ids = new Set();
	for (var i = 1; i <= 5; i++) {
		var id = require("./files/file" + i + ".js");
		expect(typeof id).toBe("string");
		expect(id.length).toBeGreaterThanOrEqual(6);
		expect(ids.has(id)).toBe(false);
		expect(id).toMatch(/^[0-9A-Za-z]+$/);
		ids.add(id);
	}
	expect(ids.size).toBe(5);
});
