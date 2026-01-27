// Test case for template literals with Unicode surrogate pairs
// This previously caused a panic: "quasic should be not empty"
// See: https://github.com/web-infra-dev/rspack/issues/12706

it("should handle template literals with Unicode surrogate pairs", () => {
	// Template literal with Unicode surrogate pair in RegExp
	const regex = new RegExp(`\uD83C[\uDFFB-\uDFFF]`, 'g');
	expect(regex).toBeInstanceOf(RegExp);
	expect(regex.source).toBe("\uD83C[\uDFFB-\uDFFF]");
	expect(regex.flags).toBe("g");

	// Test the regex actually works
	const testString = "👋🏻"; // U+1F44B U+1F3FB (waving hand with light skin tone)
	const matches = testString.match(regex);
	expect(matches).toBeTruthy();
});

it("should handle template literals with various escape sequences", () => {
	// Regular template literal
	const normal = `hello world`;
	expect(normal).toBe("hello world");

	// Template literal with hex escapes
	const withHex = `\x48\x65\x6C\x6C\x6F`;
	expect(withHex).toBe("Hello");

	// Template literal with unicode escapes
	const withUnicode = `\u0048\u0065\u006C\u006C\u006F`;
	expect(withUnicode).toBe("Hello");

	// Template literal with surrogate pairs
	const emoji = `\uD83D\uDE00`;
	expect(emoji).toBe("😀");
});

it("should handle template literals with dynamic expressions", () => {
	const value = "test";
	const result = `prefix-${value}-suffix`;
	expect(result).toBe("prefix-test-suffix");

	// With surrogate pairs and expressions
	const pattern = `\uD83C[\uDFFB-\uDFFF]`;
	const regex = new RegExp(`${pattern}`, 'g');
	expect(regex.source).toBe("\uD83C[\uDFFB-\uDFFF]");
});
