const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", done => {
  import("./use-style.js").then(({ default: x }) => {
			try {
				expect(typeof x.class).toBe("string");
				expect(x.class.length).toBeGreaterThan(0);
				expect(typeof x.local).toBe("string");
				expect(x.local.split(" ").length).toBe(4);
				expect(typeof x.cssModuleWithCustomFileExtension).toBe("string");
				expect(x.cssModuleWithCustomFileExtension.length).toBeGreaterThan(0);
				expect(x.notAValidCssModuleExtension).toBe(true);
				expect(typeof x.UsedClassName).toBe("string");
				expect(x.UsedClassName.length).toBeGreaterThan(0);

				const fs = __non_webpack_require__("fs");
				const path = __non_webpack_require__("path");
			if (__STATS_I__ === 0 || __STATS_I__ === 1) {
				let cssOutputFilename;
				if (prod) {
					const files = fs.readdirSync(__dirname);
					cssOutputFilename = files.find(f => /^\d+\.bundle1\.css$/.test(f));
					if (!cssOutputFilename) {
						throw new Error(
							`No production CSS chunk matching /^\\d+\\.bundle1\\.css$/ found in ${__dirname}. Files: ${files.join(", ")}`
						);
					}
				} else {
					cssOutputFilename = `use-style_js.bundle${__STATS_I__}.css`;
				}

					const cssContent = fs.readFileSync(
						path.join(__dirname, cssOutputFilename),
						"utf-8"
					);
					expect(cssContent).toContain(x.class);
					expect(cssContent).toContain("color: red");
					expect(cssContent).toContain("@keyframes");
				}
			} catch (e) {
			return done(e);
		}
		done();
	}, done);
});
