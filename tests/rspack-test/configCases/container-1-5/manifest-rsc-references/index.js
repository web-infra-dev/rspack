const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const manifestPath = path.join(__dirname, "mf-manifest.json");
const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

it("should capture RSC references for shared modules by package key", () => {
	const shared = stats.shared.find(item => item.name === "shared-rsc");
	expect(shared).toBeDefined();
	expect(shared.shareKey).toBe("rsc-shared-key");
	expect(shared.rsc).toBeDefined();
	expect(shared.rsc.lookup).toBe(shared.shareKey);
	expect(shared.usedIn).toContain("rsc-consumer.js");
	expect(shared.rsc.moduleType).toBe("client");
	expect(shared.rsc.clientReferences).toEqual(
		expect.arrayContaining(["SharedClientComponent", "sharedAction", "sharedValue"])
	);
	expect(shared.rsc.serverActions.length).toBeGreaterThan(0);
});

it("should capture RSC references for exposes by remoteName/exposeKey", () => {
	const expose = stats.exposes.find(item => item.path === "./button");
	expect(expose).toBeDefined();
	expect(expose.rsc).toBeDefined();
	expect(expose.rsc.lookup).toBe("container/button");
	expect(expose.rsc.moduleType).toBe("client");
	expect(expose.rsc.clientReferences).toEqual(
		expect.arrayContaining(["default", "exposedAction"])
	);
	expect(expose.requires).toContain("shared-rsc");
	expect(expose.rsc.serverActions.length).toBeGreaterThan(0);
});

it("should capture RSC context for remote module consumption", () => {
	const remoteButton = stats.remotes.find(
		item => item.alias === "@remote/alias" && item.moduleName === "Button"
	);
	expect(remoteButton).toBeDefined();
	expect(remoteButton.rsc).toBeDefined();
	expect(remoteButton.rsc.lookup).toBe("@remote/alias/Button");
	expect(remoteButton.rsc.moduleType).toBe("server");
});

it("should persist RSC metadata in mf-manifest.json", () => {
	expect(manifest.shared).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				name: "shared-rsc",
				shareKey: "rsc-shared-key",
				rsc: expect.objectContaining({
					lookup: "rsc-shared-key"
				})
			})
		])
	);
	expect(manifest.exposes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				path: "./button",
				rsc: expect.objectContaining({
					lookup: "container/button"
				})
			})
		])
	);
	expect(manifest.remotes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				alias: "@remote/alias",
				moduleName: "Button",
				rsc: expect.objectContaining({
					lookup: "@remote/alias/Button"
				})
			})
		])
	);
});
