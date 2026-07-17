import value from "./dep";

const fs = require("fs");

let typeofGuardMatched = false;
let webpackHotTypeofGuardMatched = false;

if (typeof import.meta.hot !== "undefined") {
	typeofGuardMatched = true;
}

if (typeof import.meta.webpackHot !== "undefined") {
	webpackHotTypeofGuardMatched = true;
}

if (import.meta.hot) {
	import.meta.hot.accept("./dep", mod => mod);
	import.meta.hot.accept(["./dep"], mods => mods);
	import.meta.hot.accept(() => {});
	import.meta.hot.accept();
}

it("should support a dedicated import.meta.hot context", () => {
	expect(value).toBe("import.meta.hot");
	expect(typeofGuardMatched).toBe(true);
	expect(webpackHotTypeofGuardMatched).toBe(true);
	expect(typeof import.meta.hot).toBe("object");
	expect(import.meta.hot).toBe(import.meta.hot);
	expect(import.meta.hot.data).toEqual({});
	expect(typeof import.meta.hot.accept).toBe("function");
	expect(typeof import.meta.hot.dispose).toBe("function");
	expect(import.meta.hot.decline).toBeUndefined();
	expect(import.meta.hot.status).toBeUndefined();
	expect(typeof import.meta.webpackHot).toBe("object");

	const source = fs.readFileSync(__filename, "utf-8");
	const typeofKeyword = ["type", "of"].join("");
	const runtimeTypeof = `${typeofKeyword} module.hot !== "undefined"`;
	expect(source.split(runtimeTypeof)).toHaveLength(2);
	expect(source).toContain(".hmrH(");
	const hotAcceptStart = source.indexOf("if (true) {");
	const hotAcceptEnd = source.indexOf("\n}", hotAcceptStart);
	const hotAcceptSource = source.slice(hotAcceptStart, hotAcceptEnd);
	expect(hotAcceptSource).not.toContain("module.hot");
	expect(hotAcceptSource).not.toContain("__webpack_module__.hot");
	expect(hotAcceptSource).not.toContain("__rspack_hmr_outdated");
});
