it("should handle Unicode escape sequences in template literals", function() {
// Test case from the issue - Unicode escape sequences in template literals
const regex = new RegExp(`\uD83C[\uDFFB-\uDFFF]`, 'g');
expect(regex).toBeInstanceOf(RegExp);
expect(regex.source).toBe('\uD83C[\uDFFB-\uDFFF]');

// Additional test with different Unicode escapes
const str = `Hello \u0041\u0042\u0043`;
expect(str).toBe('Hello ABC');

// Test with mixed content
const mixed = `\uD83D\uDE00 emoji`;
expect(mixed).toBe('😀 emoji');
});
