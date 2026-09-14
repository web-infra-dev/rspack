import libValue from "test_lib";
import testValue from "./node_modules/.test";

it("should snapshot managed-paths work", async () => {
	if (COMPILER_INDEX == 0) {
		expect(libValue).toBe(1);
		expect(testValue).toBe(1);
		await NEXT_HMR();
		expect(libValue).toBe(2);
		expect(testValue).toBe(2);
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(libValue).toBe(2);
		expect(testValue).toBe(3);
		await NEXT_HMR();
		expect(libValue).toBe(4);
		expect(testValue).toBe(4);
	}
});

module.hot.accept("test_lib");
module.hot.accept("./node_modules/.test");
