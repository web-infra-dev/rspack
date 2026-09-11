import { getName } from "./file";

it("should refactorize dep works", async () => {
	if (COMPILER_INDEX == 0) {
		expect(getName()).toBe("data1");
		await NEXT_HMR();
		expect(getName()).toBe("data2");
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(getName()).toBe("data2");
	}
});

module.hot.accept("./file");
