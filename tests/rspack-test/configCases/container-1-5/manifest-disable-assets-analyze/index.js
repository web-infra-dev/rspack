const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const manifestPath = path.join(__dirname, "mf-manifest.json");

const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

it("should still emit the remote entry", () => {
	const remoteEntryFile = stats.metaData.remoteEntry.name;
	const remoteEntryPath = path.join(__dirname, remoteEntryFile);
	expect(fs.existsSync(remoteEntryPath)).toBe(true);
});

it("should omit asset details from stats when disableAssetsAnalyze is true", () => {
	expect(stats.shared).toHaveLength(1);
	expect(stats.shared[0].assets.js.sync).toEqual([]);
	expect(stats.shared[0].assets.js.async).toEqual([]);
	expect(stats.exposes).toHaveLength(1);
	expect(stats.exposes[0].assets.js.sync).toEqual([]);
	expect(stats.exposes[0].assets.js.async).toEqual([]);
});

it("should omit asset details from manifest when disableAssetsAnalyze is true", () => {
	expect(manifest.shared).toHaveLength(1);
	expect(manifest.shared[0].assets.js.sync).toEqual([]);
	expect(manifest.shared[0].assets.js.async).toEqual([]);
	expect(manifest.exposes).toHaveLength(1);
	expect(manifest.exposes[0].assets.js.sync).toEqual([]);
	expect(manifest.exposes[0].assets.js.async).toEqual([]);
});

it("should still allow additionalData to augment manifest", () => {
	expect(manifest.extra).toBe(true);
});

it("should mark remote usage locations as UNKNOWN", () => {
	expect(stats.remotes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				usedIn: ["UNKNOWN"]
			})
		])
	);
});
