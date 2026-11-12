import a from "./a";
import b from "./b";
import value from "./file";

const added = typeof ADDED === "undefined" ? "missing" : ADDED;
const nested = typeof NESTED === "undefined" ? "missing" : NESTED.value;
const optionalType = typeof OPTIONAL;

it("should track definition values and keys across plugin instances", async () => {
	expect(value).toBe(Math.min(COMPILER_INDEX + 1, 4));
	expect(a).toBe(COMPILER_INDEX >= 1 ? 1 : 0);
	expect(b).toBe(COMPILER_INDEX >= 2 ? 1 : 0);
	expect(added).toBe(COMPILER_INDEX >= 4 && COMPILER_INDEX <= 6 ? "added" : "missing");
	expect(nested).toBe(COMPILER_INDEX >= 5 && COMPILER_INDEX <= 6 ? "nested" : "missing");
	expect(optionalType).toBe(COMPILER_INDEX >= 5 && COMPILER_INDEX <= 6 ? "number" : "undefined");
	if (COMPILER_INDEX < 8) await NEXT_START();
});
