const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should treeshake ui-lib correctly", async () => {
	const { Button } = await import("ui-lib");
	expect(Button).toEqual('Button');

	const bundlePath = path.join(
		__dirname,
		"node_modules_ui-lib_index_js.js"
	);
	const bundleContent = fs.readFileSync(bundlePath, "utf-8");
	expect(bundleContent).toContain("Button");
	expect(bundleContent).not.toContain("List");
});

it("should treeshake ui-lib2 correctly", async () => {
	const uiLib2 = await import("ui-lib2");
	expect(uiLib2.List).toEqual('List');

	const bundlePath = path.join(
		__dirname,
		"node_modules_ui-lib2_index_js.js"
	);
	const bundleContent = fs.readFileSync(bundlePath, "utf-8");
	expect(bundleContent).toContain("List");
	expect(bundleContent).not.toContain("Button");
	expect(bundleContent).not.toContain("Badge");
});

it("should not treeshake ui-lib-side-effect if not set sideEffect:false ", async () => {
	const uiLibSideEffect = await import("ui-lib-side-effect");
	expect(uiLibSideEffect.List).toEqual('List');

	const bundlePath = path.join(
		__dirname,
		"node_modules_ui-lib-side-effect_index_js.js"
	);
	const bundleContent = fs.readFileSync(bundlePath, "utf-8");
	expect(bundleContent).toContain("List");
	expect(bundleContent).toContain("Button");
	expect(bundleContent).toContain("Badge");
});

it("should inject usedExports into entry chunk by default", async () => {
	expect(__webpack_require__.federation.usedExports['ui-lib']['main'].sort()).toEqual([ 'Badge', 'Button' ])
});

it("should inject usedExports into manifest and stats if enable manifest", async () => {
	const { Button } = await import("ui-lib");
	expect(Button).toEqual('Button');

	const manifestPath = path.join(
		__dirname,
		"mf-manifest.json"
	);
	const manifestContent = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));
	expect(JSON.stringify(manifestContent.shared.find(s=>s.name === 'ui-lib').usedExports.sort())).toEqual(JSON.stringify([
		"Badge",
		"Button"
	]));

		const statsPath = path.join(
		__dirname,
		"mf-stats.json"
	);
	const statsContent = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
	expect(JSON.stringify(statsContent.shared.find(s=>s.name === 'ui-lib').usedExports.sort())).toEqual(JSON.stringify([
		"Badge",
		"Button"
	]));
});

// it("should ")
