const cssUrl = new URL("./target.css", import.meta.url);

it(`should reference an emitted stylesheet in ${CONFIG_NAME}`, () => {
	const fs = require("fs");
	const path = require("path");
	const cssFiles = fs.readdirSync(OUTPUT_DIR).filter(name => name.endsWith(".css"));
	if (EXTRACT_CSS) {
		expect(cssFiles).toContain(`${CONFIG_NAME}-shared.css`);
		const sharedCss = fs.readFileSync(
			path.join(OUTPUT_DIR, `${CONFIG_NAME}-shared.css`),
			"utf-8"
		);
		expect(sharedCss).toContain(".url-entry-target");
		expect(sharedCss).toContain(".url-entry-imported");
	}

	const filename = path.basename(cssUrl.pathname);
	if (EXTRACT_CSS) {
		expect(filename).toBe(`${CONFIG_NAME}-shared.css`);
	}
	expect(cssFiles).toContain(filename);
	const css = fs.readFileSync(path.join(OUTPUT_DIR, filename), "utf-8");
	expect(css).toContain(".url-entry-target");
	expect(css).toContain(".url-entry-imported");
});
