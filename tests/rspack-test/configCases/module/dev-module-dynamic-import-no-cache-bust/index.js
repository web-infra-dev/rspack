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
	expect(source).not.toContain('?t=" + Date.now()');
});

it("should still load dynamic import", async () => {
	const runtimeChunk = __STATS__.chunks.find(chunk =>
		chunk.names.includes("runtime~main")
	);
	expect(runtimeChunk).toBeDefined();

	const isOnlyRuntimeAndMain = __STATS__.chunks.every(chunk =>
		chunk.names.includes("runtime~main") || chunk.names.includes("main")
	);
	expect(isOnlyRuntimeAndMain).toBe(true);
});
