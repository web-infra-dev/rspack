it("should have unique hashed module ids with custom options", function () {
	var ids = new Set();
	for (var i = 1; i <= 5; i++) {
		var id = require("./files/file" + i + ".js");
		expect(typeof id).toBe("string");
		expect(id.length).toBeGreaterThanOrEqual(1);
		expect(ids.has(id)).toBe(false);
		expect(Number.isNaN(Number(id))).toBe(true);
		ids.add(id);
	}
	expect(ids.size).toBe(5);
});

it("should assign a hashed id to the entry module", function () {
	var entryId = module.id;
	expect(typeof entryId).toBe("string");
	expect(entryId.length).toBeGreaterThanOrEqual(1);
	expect(Number.isNaN(Number(entryId))).toBe(true);
});

it("should produce deterministic ids", function () {
	var id1 = require("./files/file1.js");
	var id2 = require("./files/file1.js");
	expect(id1).toBe(id2);
});
