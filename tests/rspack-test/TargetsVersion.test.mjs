import {
	decodeVersion,
	encodeVersion,
} from "../../packages/rspack/src/util/targetsVersion.ts";

describe("targetsVersion", () => {
	it("should round-trip valid browser versions", () => {
		expect(decodeVersion(encodeVersion("120"))).toBe("120.0.0");
		expect(decodeVersion(encodeVersion("16.4-16.6"))).toBe("16.4.0");
		expect(decodeVersion(encodeVersion("1.2.3"))).toBe("1.2.3");
	});

	it("should return null for out-of-range or negative version components", () => {
		expect(encodeVersion("1.256.0")).toBeNull();
		expect(encodeVersion("1.0.256")).toBeNull();
		expect(encodeVersion("65536.0.0")).toBeNull();
		expect(encodeVersion("1.-1.0")).toBeNull();
		expect(encodeVersion("invalid")).toBeNull();
	});
});
