const fs = require("fs");

globalThis.stringDeduplicationResult = [
	"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
	__STRING_DEDUPLICATION_FILLER__,
	"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
].filter(value => value.length < 100);

it("deduplicates repeated strings with a compact binding", () => {
	const repeated = globalThis.stringDeduplicationResult[0];
	expect(globalThis.stringDeduplicationResult).toEqual([repeated, repeated]);

	const source = fs.readFileSync(__filename, "utf-8");
	expect(source.split(repeated)).toHaveLength(2);
	expect(source).toContain("const $=");
});
