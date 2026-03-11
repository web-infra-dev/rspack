import { pkg, bin } from "./shared";
import { name, version, rest } from "./shared2";

it("should preserve json properties used via an exported require result", () => {
	const { name, version } = pkg;

	expect(bin).toBe("cli");
	expect(name).toBe("rspack-cjs-treeshaking");
	expect(version).toBe("1.0.0");
});

it("should preserve destructured exports from require", () => {
	expect(name).toBe("rspack-cjs-treeshaking");
	expect(version).toBe("1.0.0");
	expect(Object.keys(rest.bin)[0]).toBe("cli");
});
