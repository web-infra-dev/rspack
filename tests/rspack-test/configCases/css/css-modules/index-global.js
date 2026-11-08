const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", done => {
  import("./use-style-global.js").then(({ default: x }) => {
		try {
			const fs = require("fs");
			const path = require("path");
			expect(x).toMatchFileSnapshotSync(
				path.join(__SNAPSHOT__, `global-classes-${prod ? "prod" : "dev"}.${__STATS_I__}.txt`)
			);

			if (__STATS_I__ === 4 || __STATS_I__ === 5) {
				let cssOutputFilename;
				if (prod) {
					const files = fs.readdirSync(__dirname);
					cssOutputFilename = files.find(f => new RegExp(`^\\d+\\.bundle${__STATS_I__}\\.css$`).test(f));
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
				expect(cssContent).toMatchFileSnapshotSync(
					path.join(__SNAPSHOT__, `global-css-${prod ? "prod" : "dev"}.${__STATS_I__}.txt`)
				);
			}
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
});
