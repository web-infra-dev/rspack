it("should define TEST_VALUE1 from file", function () {
	expect(TEST_VALUE1).toBe("test-value-123");
	expect(typeof TEST_VALUE1).toBe("string");
});

it("should define TEST_VALUE2 with static runtime value", function () {
	expect(TEST_VALUE2).toBe("static-runtime-value");
	expect(typeof TEST_VALUE2).toBe("string");
});

it("should define TEST_VALUE3 with uncacheable value", function () {
	expect(typeof TEST_VALUE3).toBe("string");
	const value = JSON.parse(TEST_VALUE3);
	expect(typeof value).toBe("number");
	expect(value).toBeGreaterThan(0);
});

it("should define TEST_VALUE4 with options object", function () {
	expect(TEST_VALUE4).toBe("test-value-123-with-options");
	expect(typeof TEST_VALUE4).toBe("string");
});

it("should define TEST_VALUE5 with version", function () {
	const parsed = JSON.parse(TEST_VALUE5);
	expect(parsed.version).toBe("1.0.0");
	expect(parsed.key).toBe("TEST_VALUE5");
});

it("should define nested values", function () {
	expect(NESTED.VALUE).toBe("nested-value");
	expect(typeof NESTED.VALUE).toBe("string");
});

it("should handle different return types", function () {
	expect(RUNTIME_NUMBER).toBe(42);
	expect(typeof RUNTIME_NUMBER).toBe("number");
	
	expect(RUNTIME_BOOLEAN).toBe(true);
	expect(typeof RUNTIME_BOOLEAN).toBe("boolean");
	
	expect(RUNTIME_NULL).toBe(null);
	
	expect(typeof RUNTIME_UNDEFINED).toBe("undefined");
});

it("should handle errors gracefully", function () {
	expect(typeof ERROR_VALUE).toBe("undefined");
});

it("should provide module context", function () {
	// In test environment, module context might not be available
	expect(MODULE_VALUE).toBe("no-module");
});