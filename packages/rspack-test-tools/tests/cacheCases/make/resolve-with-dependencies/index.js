import num from "./file";
import "./a.js";

it("should clean isolated cycle", async () => {
	if (COMPILER_INDEX == 0) {
		expect(num).toBe(1);
		await NEXT_HMR();
		expect(num).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(num).toBe(3);
		await NEXT_HMR();
		expect(num).toBe(4);
	}
});
