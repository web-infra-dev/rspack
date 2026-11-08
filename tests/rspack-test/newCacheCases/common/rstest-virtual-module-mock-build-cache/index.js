import value from "virtual-module";

rs.mock("virtual-module", () => ({
	default: "mocked-virtual-module",
}));

it("should keep virtual module mock hoists working with persistent cache", async () => {
	expect(value).toBe("mocked-virtual-module");

	if (COMPILER_INDEX === 0) {
		await NEXT_START();
	}

	if (COMPILER_INDEX === 1) {
		expect(value).toBe("mocked-virtual-module");
	}
});
---
import value from "virtual-module";

rs.mock("virtual-module", () => ({
	default: "mocked-virtual-module",
}));

it("should keep virtual module mock hoists working with persistent cache", async () => {
	expect(value).toBe("mocked-virtual-module");

	if (COMPILER_INDEX === 0) {
		await NEXT_START();
	}

	if (COMPILER_INDEX === 1) {
		expect(value).toBe("mocked-virtual-module");
	}
});
