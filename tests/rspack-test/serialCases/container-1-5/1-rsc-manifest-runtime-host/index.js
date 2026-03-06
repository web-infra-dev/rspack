const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

function readJson(filePath) {
	return JSON.parse(fs.readFileSync(filePath, "utf-8"));
}

it("should emit host and remote RSC metadata across serial MF builds", async () => {
	const hostStats = readJson(path.join(__dirname, "mf-stats.json"));
	const hostManifest = readJson(path.join(__dirname, "mf-manifest.json"));
	const remoteDir = path.resolve(__dirname, "../0-rsc-manifest-runtime-remote");
	const remoteStats = readJson(path.join(remoteDir, "mf-stats.json"));
	const remoteManifest = readJson(path.join(remoteDir, "mf-manifest.json"));

	expect(hostStats.name).toBe("host");
	expect(hostManifest.name).toBe("host");
	expect(remoteStats.name).toBe("remote");
	expect(remoteManifest.name).toBe("remote");

	const hostShared = hostStats.shared.find(item => item.name === "shared-rsc");
	expect(hostShared).toBeDefined();
	expect(hostShared.shareKey).toBe("rsc-shared-key");
	expect(hostShared.rsc.lookup).toBe("rsc-shared-key");

	const hostExpose = hostStats.exposes.find(item => item.path === "./button");
	expect(hostExpose).toBeDefined();
	expect(hostExpose.rsc.lookup).toBe("host/button");
	expect(hostExpose.rsc.serverActions.length).toBeGreaterThan(0);

	const hostConsumerExpose = hostStats.exposes.find(item => item.path === "./consumer");
	expect(hostConsumerExpose).toBeDefined();
	expect(hostConsumerExpose.rsc.lookup).toBe("host/consumer");
	expect(hostConsumerExpose.rsc.serverActions.length).toBeGreaterThan(0);

	const remoteButton = hostStats.remotes.find(
		item => item.alias === "@remote/alias" && item.moduleName === "Button"
	);
	expect(remoteButton).toBeDefined();
	expect(remoteButton.rsc.lookup).toBe("@remote/alias/Button");
	expect(remoteButton.rsc.moduleType).toBe("server");

	const remoteManifestExpose = remoteManifest.exposes.find(
		item => item.path === "./Button"
	);
	expect(remoteManifestExpose).toBeDefined();
	expect(remoteManifestExpose.rsc.lookup).toBe("remote/Button");

	const loadedRemoteModule = await import("@remote/alias/Button");
	expect(loadedRemoteModule).toBeDefined();
	expect(Object.keys(loadedRemoteModule).length).toBeGreaterThan(0);
});
