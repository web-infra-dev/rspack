function getNestedValue(value) {
	function __rspack_context(input) {
		return input + value;
	}
	return __rspack_context(40);
}

it("keeps nested __rspack_context binding local", () => {
	expect(getNestedValue(2)).toBe(42);
});

it("renames nested __rspack_context in rspack runtime mode", () => {
	if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		return;
	}
	const source = require("fs").readFileSync(__filename, "utf-8");
	expect(source).toContain("__nested_rspack_require_");
	expect(source).not.toMatch(
		new RegExp(["function ", "__rspack", "_context", "\\("].join(""))
	);
});
