import a from "./a";
import b from "./b";
import value from "./file";

it("should define plugin works", async () => {
	if (COMPILER_INDEX === 0) {
		expect(value).toBe(1);
		expect(a).toBe(0);
		expect(b).toBe(0);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 1) {
		expect(value).toBe(2);
		expect(a).toBe(1);
		expect(b).toBe(0);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 2) {
		expect(value).toBe(3);
		expect(a).toBe(1);
		expect(b).toBe(1);
		await NEXT_START();
	}
	if (COMPILER_INDEX === 3) {
		expect(value).toBe(4);
		expect(a).toBe(1);
		expect(b).toBe(1);
	}
});
