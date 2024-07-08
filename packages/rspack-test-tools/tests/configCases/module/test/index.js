import { lib } from "./lib";

it("`test` should work well with both `string` and `regex`", () => {
	expect(lib).toEqual(44);
});
it("support js regex", () => {
	const cssModule = require("./a.module.less");
	const css = require("./a.less");
	expect(css).toEqual(nsObj({}));
	expect(cssModule["module-test"]).toBeTruthy;
});

it("should support regex flags", () => {
	const svg = require("./a.SVG");
	expect(svg.startsWith("data:image/svg+xml;base64,")).toBe(true);
});
