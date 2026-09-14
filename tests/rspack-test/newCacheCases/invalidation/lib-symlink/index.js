import { version } from "tools";
import value from "./file";

it("should invalidation work when using lib symlink", async () => {
	if (COMPILER_INDEX == 0) {
		expect(value).toBe(1);
		expect(version).toBe("1.0.0");
		await NEXT_START();
	}
	if (COMPILER_INDEX == 1) {
		expect(value).toBe(1);
		expect(version).toBe("2.0.0");
	}
});
