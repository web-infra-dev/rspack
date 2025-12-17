const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const manifestPath = path.join(__dirname, "mf-manifest.json");
const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

it("should emit remote entry with hash", () => {
	const remoteEntryFile = stats.metaData.remoteEntry.name;
	const remoteEntryPath = path.join(__dirname, remoteEntryFile);
	expect(fs.existsSync(remoteEntryPath)).toBe(true);
});

it("should report xreact shared assets in sync only", () => {
    const xreact = stats.shared.find(item => item.name === "xreact");
		expect(xreact.singleton).toBe(true);
    expect(xreact).toBeDefined();
    expect(xreact.assets.css.sync).toEqual([]);
    expect(xreact.assets.css.async).toEqual([]);
    expect(xreact.assets.js.sync).toEqual(["node_modules_xreact_index_js.js"]);
    expect(xreact.assets.js.async).toEqual([]);
});

it("should include scoped shared '@scope-sc/dep1' in stats", () => {
    const dep1 = stats.shared.find(item => item.name === "@scope-sc/dep1");
    expect(dep1).toBeDefined();
		expect(dep1.singleton).toBe(true);
    expect(dep1.version).toBe("1.0.0");
    expect(dep1.requiredVersion).toBe("^1.0.0");
    expect(dep1.assets.css.sync).toEqual([]);
    expect(dep1.assets.css.async).toEqual([]);
    expect(Array.isArray(dep1.assets.js.sync)).toBe(true);
    expect(dep1.assets.js.async).toEqual([]);
});

it("should include scoped shared '@scope-sc2/dep2' in stats", () => {
    const dep2 = stats.shared.find(item => item.name === "@scope-sc2/dep2");
    expect(dep2).toBeDefined();
		expect(dep2.singleton).toBe(false);
    expect(dep2.version).toBe("1.0.0");
    expect(dep2.requiredVersion).toBe(">=1.0.0");
    expect(dep2.assets.css.sync).toEqual([]);
    expect(dep2.assets.css.async).toEqual([]);
    expect(Array.isArray(dep2.assets.js.sync)).toBe(true);
    expect(dep2.assets.js.async).toEqual([]);
    expect(dep2.usedIn.includes("module.js")).toBe(true);
});


//exposes
it("should expose sync assets only", () => {
	expect(stats.exposes).toHaveLength(1);
	expect(stats.exposes[0].file).toBe("module.js");
	expect(stats.exposes[0].assets.js.sync).toEqual(["_federation_expose_a.js"]);
	expect(stats.exposes[0].assets.js.async).toEqual([
		"lazy-module_js.js"
	]);
});

it("should reflect expose assets in manifest", () => {
	expect(manifest.exposes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				name: "expose-a",
				path: "./expose-a",
				assets: expect.objectContaining({
					js: expect.objectContaining({
						sync: ["_federation_expose_a.js"],
						async: [
							"lazy-module_js.js",
						]
					})
				})
			})
		])
	);
});

// remotes

it("should record remote usage", () => {
    expect(stats.remotes).toEqual(
        expect.arrayContaining([
            // actual remote usage recorded for a concrete module
            expect.objectContaining({
                alias: "@remote/alias",
                consumingFederationContainerName: "container",
                federationContainerName: "remote",
                moduleName: "Button",
                usedIn: expect.arrayContaining(["module.js"]),
                entry: 'http://localhost:8000/remoteEntry.js'
            }),
            // ensured default remote record with moduleName "."
            expect.objectContaining({
                alias: "@remote/alias",
                consumingFederationContainerName: "container",
                federationContainerName: "remote",
                moduleName: ".",
                usedIn: expect.arrayContaining(["module.js"]),
                entry: 'http://localhost:8000/remoteEntry.js'
            }),
            // dynamic remote ensured with default values
            expect.objectContaining({
                alias: "dynamic-remote",
                consumingFederationContainerName: "container",
                federationContainerName: "dynamic_remote",
                moduleName: ".",
                usedIn: expect.arrayContaining([
                    "UNKNOWN"
                ]),
                entry: 'http://localhost:8001/remoteEntry.js'
            })
        ])
    );
});

it("should persist remote metadata in manifest", () => {
    expect(manifest.remotes).toEqual(
        expect.arrayContaining([
            expect.objectContaining({
                alias: "@remote/alias",
                federationContainerName: "remote",
                moduleName: "."
            }),
            expect.objectContaining({
                alias: "dynamic-remote",
                federationContainerName: "dynamic_remote",
                moduleName: "."
            })
        ])
    );
});
