// Prettier and biome add a trailing comma to multi-line calls by default, so
// published packages ship `require.resolve` calls that look like this.
it("should handle require.resolve with a trailing comma", function () {
	const id = require.resolve(
		"./foo.js",
	);
	expect(id).toBe(require.resolve("./foo.js"));
});

it("should handle require.resolve with a trailing comma and a comment", function () {
	const id = require.resolve("./bar.js" /* a, b */,);
	expect(id).toBe(require.resolve("./bar.js"));
});

it("should handle require.resolveWeak with a trailing comma", function () {
	const id = require.resolveWeak(
		"./foo.js",
	);
	expect(id).toBe(require.resolveWeak("./foo.js"));
});

it("should handle a context require.resolve with a trailing comma", function () {
	function getFile() {
		return "foo";
	}

	const id = require.resolve(
		"./dir/" + getFile() + ".js",
	);
	expect(id).toBe(require.resolve("./dir/" + getFile() + ".js"));
});

it("should handle import.meta.resolve with a trailing comma", function () {
	const id = import.meta.resolve(
		"./foo.js",
	);
	expect(id).toBe(require.resolve("./foo.js"));
});
