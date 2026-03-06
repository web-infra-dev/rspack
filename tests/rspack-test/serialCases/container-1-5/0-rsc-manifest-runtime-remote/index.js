const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

function readJson(fileName) {
	return JSON.parse(fs.readFileSync(path.join(__dirname, fileName), "utf-8"));
}

it("should emit remote RSC manifest data in a standalone build", () => {
	const stats = readJson("mf-stats.json");
	const manifest = readJson("mf-manifest.json");

	expect(stats.name).toBe("remote");
	expect(manifest.name).toBe("remote");
	expect(stats.remotes).toEqual([]);
	expect(manifest.remotes).toEqual([]);

	const shared = stats.shared.find(item => item.name === "shared-rsc");
	expect(shared).toBeDefined();
	expect(shared.shareKey).toBe("rsc-shared-key");
	expect(shared.rsc.lookup).toBe("rsc-shared-key");
	expect(shared.rsc.clientReferences).toEqual(
		expect.arrayContaining(["SharedClientComponent", "sharedAction", "sharedValue"])
	);
	expect(shared.rsc.serverActions.length).toBeGreaterThan(0);

	const buttonExpose = stats.exposes.find(item => item.path === "./Button");
	expect(buttonExpose).toBeDefined();
	expect(buttonExpose.rsc.lookup).toBe("remote/Button");
	expect(buttonExpose.rsc.clientReferences).toEqual(
		expect.arrayContaining(["default", "remoteAction"])
	);
	expect(buttonExpose.rsc.serverActions.length).toBeGreaterThan(0);

	const consumerExpose = stats.exposes.find(item => item.path === "./Consumer");
	expect(consumerExpose).toBeDefined();
	expect(consumerExpose.rsc.lookup).toBe("remote/Consumer");
	expect(consumerExpose.rsc.serverActions.length).toBeGreaterThan(0);

	const manifestShared = manifest.shared.find(item => item.name === "shared-rsc");
	expect(manifestShared).toEqual(
		expect.objectContaining({
			name: "shared-rsc",
			shareKey: "rsc-shared-key",
			rsc: expect.objectContaining({
				lookup: "rsc-shared-key"
			})
		})
	);
});
