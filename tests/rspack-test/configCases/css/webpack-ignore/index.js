import "./style.css";

it("should preserve URLs selected by webpackIgnore", () => {
	const links = document.getElementsByTagName("link");
	const css = links[1].sheet.css;

	// The last applicable comment wins when multiple comments precede a dependency.
	expect(css).toContain(
		"@import /* webpackIgnore: false */ /* webpackIgnore: true */ url(./basic.css);"
	);
	// Cover quoted and unquoted URL functions.
	expect(css).toMatch(
		/\/\* webpackIgnore: true \*\/\s*url\("\.\/url\/img\.png"\)/
	);
	expect(css).toMatch(
		/\/\*\s*webpackIgnore:\s+true\s*\*\/\s*url\(\.\/url\/img\.png\)/
	);
	// image-set() accepts both URL functions and string image candidates.
	expect(css).toMatch(
		/image-set\([\s\S]*?\/\*webpackIgnore:\s+true\*\/[\s\S]*?url\(\.\/url\/img\.png\) 2x/
	);
	expect(css).toMatch(
		/image-set\([\s\S]*?\/\*webpackIgnore:\s+true\*\/[\s\S]*?'\.\/url\/img\.png' 2x/
	);
});
