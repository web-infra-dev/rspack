import { pkg, bin } from "./shared";

it("should preserve json properties used via an exported require result", () => {
	const { name, version } = pkg;

	expect(bin).toBe("cli");
	expect(name).toBe("rspack-cjs-treeshaking");
	expect(version).toBe("1.0.0");
});
