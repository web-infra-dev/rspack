const fs = require("fs");
const path = require("path");

it("should emit manifest assets with forward-slash names for a Windows-style filePath", () => {
	const assetNames = __STATS__.assets.map(asset => asset.name);
	expect(assetNames).toContain("custom/path/mf-manifest.json");
	expect(assetNames).toContain("custom/path/mf-stats.json");
	for (const name of assetNames) {
		expect(name).not.toContain("\\");
	}
});

it("should write valid manifest files to the forward-slash location", () => {
	const manifestPath = path.join(
		__dirname,
		"custom",
		"path",
		"mf-manifest.json"
	);
	const statsPath = path.join(__dirname, "custom", "path", "mf-stats.json");
	expect(fs.existsSync(manifestPath)).toBe(true);
	expect(fs.existsSync(statsPath)).toBe(true);
	const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));
	const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
	expect(manifest.name).toBe("container");
	expect(stats.name).toBe("container");
});
