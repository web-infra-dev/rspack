const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const customPath = 'custom-path';
const customManifestPath = path.join(__dirname, customPath,"custom-manifest.json");
const customStatsPath = path.join(__dirname, customPath,"custom-manifest-stats.json");
const defaultManifestPath = path.join(__dirname, "mf-manifest.json");
const defaultStatsPath = path.join(__dirname, "mf-stats.json");

const stats = JSON.parse(fs.readFileSync(customStatsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(customManifestPath, "utf-8"));

it("should emit manifest with the configured fileName", () => {
	expect(fs.existsSync(customManifestPath)).toBe(true);
	expect(fs.existsSync(customStatsPath)).toBe(true);
});

it("should not emit default manifest file names when fileName is set", () => {
	expect(fs.existsSync(defaultManifestPath)).toBe(false);
	expect(fs.existsSync(defaultStatsPath)).toBe(false);
});

it("should still point to the emitted remote entry", () => {
	const remoteEntryFile = stats.metaData.remoteEntry.name;
	const remoteEntryPath = path.join(__dirname, remoteEntryFile);
	expect(fs.existsSync(remoteEntryPath)).toBe(true);
});
