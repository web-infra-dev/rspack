import { formatSize } from "../../packages/rspack/src/util/SizeFormatHelpers.ts";

describe("SizeFormatHelpers.formatSize", () => {
	it("should format standard byte ranges", () => {
		expect(formatSize(0)).toBe("0 bytes");
		expect(formatSize(-10)).toBe("0 bytes");
		expect(formatSize(512)).toBe("512 bytes");
		expect(formatSize(1024)).toBe("1 KiB");
		expect(formatSize(1024 ** 2)).toBe("1 MiB");
		expect(formatSize(1024 ** 3)).toBe("1 GiB");
	});

	it("should handle non-finite values as unknown size", () => {
		expect(formatSize(NaN)).toBe("unknown size");
		expect(formatSize(Number.POSITIVE_INFINITY)).toBe("unknown size");
		expect(formatSize(Number.NEGATIVE_INFINITY)).toBe("unknown size");
		expect(formatSize(undefined)).toBe("unknown size");
	});

	it("should clamp unit index for fractional bytes and sizes >= 1 TiB", () => {
		expect(formatSize(0.5)).toBe("0.5 bytes");
		expect(formatSize(1024 ** 4)).toBe("1020 GiB");
	});
});
