const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", async () => {
	const { default: x } = await import("./use-style.js");

	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	expect(x).toEqual(
		prod
			? {
					UsedClassName: "my-app-744-GW",
					cssModuleWithCustomFileExtension: "my-app-666-pQ",
					exportLocalVarsShouldCleanup: "false false",
					notAValidCssModuleExtension: true
				}
			: {
					UsedClassName: "./identifiers.module.css-UsedClassName",
					cssModuleWithCustomFileExtension: "./style.module.my-css-myCssClass",
					exportLocalVarsShouldCleanup: "false false",
					notAValidCssModuleExtension: true
				}
	);
	if (
		typeof __STATS_I__ === "undefined" ||
		__STATS_I__ === 0 ||
		__STATS_I__ === 1
	) {
		let cssOutputFilename;
		if (prod) {
			const files = fs.readdirSync(__dirname);
			cssOutputFilename = files.find(f => /^\d+\.bundle1\.css$/.test(f));
			if (!cssOutputFilename) {
				throw new Error(
					`No production CSS chunk matching /^\\d+\\.bundle1\\.css$/ found in ${__dirname}. Files: ${files.join(", ")}`
				);
			}
		} else if (typeof __STATS_I__ === "undefined") {
			cssOutputFilename = "use-style_js.bundle0.css";
		} else {
			cssOutputFilename = `use-style_js.bundle${__STATS_I__}.css`;
		}

		const cssContent = fs.readFileSync(
			path.join(__dirname, cssOutputFilename),
			"utf-8"
		);
		expect(`${cssContent}\n`).toMatchFileSnapshotSync(
			path.join(__SNAPSHOT__, `cssContent.${prod ? "prod" : "dev"}.txt`)
		);
	}
});
