const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const manifestPath = path.join(__dirname, "mf-manifest.json");
const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

it("should record configured and custom usedExports for shared module", () => {
	const sharedStats = stats.shared.find(item => item.name === "react");
	const sharedManifest = manifest.shared.find(item => item.name === "react");
	const expected = ["Button"];
	expect(sharedStats.usedExports.sort()).toEqual(expected);
	expect(sharedManifest.usedExports.sort()).toEqual(expected);
});
