import { value } from "shared-module";

it("should split vendors despite maxInitialRequests=1 because enforceSizeThreshold is exceeded", () => {
	// When enforceSizeThreshold is exceeded, maxInitialRequests should be ignored.
	// This means the shared module should be split into a separate "vendors" chunk.
	const chunkNames = __STATS__.chunks.map(c => c.names).flat();
	expect(chunkNames).toContain("vendors");
	expect(value).toBe(42);
});
