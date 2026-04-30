it("should not include development cache bust in production", () => {
	const fs = require("fs");
	const path = require("path");
	const runtimeChunk = __STATS__.chunks.find(chunk =>
		chunk.names.includes("runtime~main")
	);

	expect(runtimeChunk).toBeDefined();

	const runtimeAsset = runtimeChunk.files.find(file => file.endsWith(".js") || file.endsWith(".mjs"));
	expect(runtimeAsset).toBeDefined();

	const source = fs.readFileSync(path.join(__dirname, runtimeAsset), "utf-8");
	expect(source).not.toContain("?t=");
});

it("should still load dynamic import", async () => {
	const mod = await import("./chunk");
	expect(mod.default).toBe(42);

	const runtimeChunk = __STATS__.chunks.find(chunk =>
		chunk.names.includes("runtime~main")
	);
	expect(runtimeChunk).toBeDefined();

	const hasAsyncChunk = __STATS__.chunks.some(chunk => {
		if (chunk.names.includes("runtime~main") || chunk.names.includes("main")) {
			return false;
		}
		return chunk.files.some(file => file.endsWith(".mjs"));
	});
	expect(hasAsyncChunk).toBe(true);
});
