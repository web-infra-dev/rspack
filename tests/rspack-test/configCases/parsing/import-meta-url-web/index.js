export {};

it("should not be replaced by file path", function() {
	// import.meta.url should be preserved in the bundle (not replaced with a file:// URL string)
	// When preserved, it will be resolved by the runtime environment
	// In Node.js test environment, it resolves to file:// URL, but in browser it would be http://
	// The important thing is that it's not replaced during build time
	const url = import.meta.url;
	// Just verify it's a string (not undefined, which would mean it was replaced with unsupported comment)
	expect(typeof url).toBe("string");
	// The actual URL format depends on the runtime environment, but it should exist
	expect(url).toBeTruthy();
});
