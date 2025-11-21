import react from "react";
import value from "./file";

it("should basic test work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe(1);
		await NEXT_HMR();
		expect(value).toBe(2);
		expect(typeof react).toBe("object");
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(3);
		await NEXT_HMR();
		expect(value).toBe(4);
		expect(typeof react).toBe("object");
	}
});

module.hot.accept("./file");