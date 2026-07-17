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

const __RSPACK_IMPORT_META_HOT_ACCEPT_START__ = true;

if (import.meta.hot) {
	import.meta.hot.accept("./dep", mod => mod);
	import.meta.hot.accept(["./dep"], mods => mods);
	import.meta.hot.accept(() => {});
	import.meta.hot.accept();
}

const __RSPACK_IMPORT_META_HOT_ACCEPT_END__ = true;

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
	const moduleHotNeedle = ["module", "hot"].join(".");
	const runtimeTypeof = `${typeofKeyword} ${moduleHotNeedle} !== "undefined"`;
	expect(source.split(runtimeTypeof)).toHaveLength(2);
	expect(__RSPACK_IMPORT_META_HOT_ACCEPT_START__).toBe(true);
	expect(__RSPACK_IMPORT_META_HOT_ACCEPT_END__).toBe(true);
	const hotAcceptStartMarker = [
		"__RSPACK_IMPORT_META_HOT_",
		"ACCEPT_START__"
	].join("");
	const hotAcceptEndMarker = [
		"__RSPACK_IMPORT_META_HOT_",
		"ACCEPT_END__"
	].join("");
	const hotContextNeedle = [".hmr", "H("].join("");
	const moduleArgumentHotNeedle = ["__webpack_module__", "hot"].join(".");
	const outdatedNeedle = ["__rspack_hmr", "outdated"].join("_");
	const hotAcceptStart = source.indexOf(hotAcceptStartMarker);
	const hotAcceptEnd = source.indexOf(hotAcceptEndMarker, hotAcceptStart);
	expect(hotAcceptStart).toBeGreaterThan(-1);
	expect(hotAcceptEnd).toBeGreaterThan(hotAcceptStart);
	const hotAcceptSource = source.slice(hotAcceptStart, hotAcceptEnd);
	expect(hotAcceptSource).toContain(hotContextNeedle);
	expect(hotAcceptSource).not.toContain(moduleHotNeedle);
	expect(hotAcceptSource).not.toContain(moduleArgumentHotNeedle);
	expect(hotAcceptSource).not.toContain(outdatedNeedle);
});
