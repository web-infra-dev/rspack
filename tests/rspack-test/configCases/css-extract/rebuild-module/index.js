const fs = require("fs");
const path = require("path");

it("should not panic when rebuilding a css module extracted with css-extract", async () => {
	await import("./render").then((exports) => {
		exports.render();
	});

	const css = fs.readFileSync(path.join(__dirname, "bundle.css"), "utf-8");
	// The loader runs twice (once in the make phase, once during the rebuild),
	// but the extracted CSS still reflects the first run: the `css|...` module
	// is reused by identifier so its content is not refreshed. This matches
	// webpack, see https://github.com/web-infra-dev/rspack/issues/9778.
	expect(css).toContain("green");
});
