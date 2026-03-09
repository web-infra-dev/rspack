it("should have unique hashed module ids", function () {
	var ids = new Set();
	for (var i = 1; i <= 5; i++) {
		var id = require("./files/file" + i + ".js");
		expect(typeof id).toBe("string");
		expect(id.length).toBeGreaterThanOrEqual(4);
		expect(ids.has(id)).toBe(false);
		expect(Number.isNaN(Number(id))).toBe(true);
		expect(id).toMatch(/^[A-Za-z0-9+/]+=*$/);
		ids.add(id);
	}
	expect(ids.size).toBe(5);
});

it("should produce the same id for the same module", function () {
	var id1 = require("./files/file1.js");
	var id2 = require("./files/file1.js");
	expect(id1).toBe(id2);
});

it("should produce different ids for different modules", function () {
	var id1 = require("./files/file1.js");
	var id2 = require("./files/file2.js");
	expect(id1).not.toBe(id2);
});