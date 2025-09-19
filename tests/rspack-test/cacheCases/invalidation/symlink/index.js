import value from "./driver";
import version from "./tool"

it("should invalidate work when symlink changes", async () => {
	if (COMPILER_INDEX == 0) {
		expect(version).toBe(100);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(version).toBe(200);
	}
});
