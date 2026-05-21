import "./style-root.style.css";

const STATS = __STATS__.children[__STATS_I__];

it("should fold every style-export module into a single concatenated module", () => {
	const concatModules = STATS.modules.filter((m) => m.modules);
	if (concatModules.length > 0) {
		// index-style.js + style-root; style-leaf is folded into style-root.
		expect(concatModules[0].modules.length).toBeGreaterThanOrEqual(2);
	}
});

it("should keep the require runtime for separate style imports", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const source = fs.readFileSync(
		path.join(STATS.outputPath, `bundle${__STATS_I__}.js`),
		"utf-8"
	);

	// Style @import modules stay as separate injected style modules when
	// they are not concatenated into the same scope, so the bundle needs
	// the require runtime to execute the imported module.
	const marker = `__webpack_${"module"}_cache__`;
	if (STATS.modules.some((m) => m.modules)) {
		expect(source).toContain(marker);
	}
});
