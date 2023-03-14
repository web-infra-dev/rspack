import { lib } from "./lib";

it("`test` should work well with both `string` and `regex`", () => {
	expect(lib).toEqual(44);
});
it("support js regex", () => {
	const cssModule = require("./a.module.less");
	const css = require("./a.less");
	expect(css).toEqual({});
	expect(cssModule["module-test"]).toBeTruthy;
});
