const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", done => {
  import("./use-style-global.js").then(({ default: x }) => {
			try {
				expect(typeof x).toBe("object");
				expect(Object.keys(x).length).toBeGreaterThan(0);

				const fs = __non_webpack_require__("fs");
				const path = __non_webpack_require__("path");
				if (__STATS_I__ === 4 || __STATS_I__ === 5) {
					let cssOutputFilename;
					if (prod) {
						const files = fs.readdirSync(__dirname);
						cssOutputFilename = files.find(f =>
							new RegExp(`^\\d+\\.bundle${__STATS_I__}\\.css$`).test(f)
						);
						if (!cssOutputFilename) {
							throw new Error(
								`No production CSS chunk matching /^\\d+\\.bundle${__STATS_I__}\\.css$/ found in ${__dirname}. Files: ${files.join(", ")}`
							);
						}
					} else {
						cssOutputFilename = `use-style-global_js.bundle${__STATS_I__}.css`;
					}

					const cssContent = fs.readFileSync(
						path.join(__dirname, cssOutputFilename),
						"utf-8"
					);
					expect(cssContent).toContain(".class");
					expect(cssContent).toContain("color: red");
				}
			} catch (e) {
			return done(e);
		}
		done();
	}, done);
});
