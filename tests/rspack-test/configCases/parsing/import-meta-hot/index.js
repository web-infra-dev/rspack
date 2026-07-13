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
	import.meta.hot.accept(["./dep"], () => {});
}

it("should support import.meta.hot as an HMR alias", () => {
	expect(value).toBe("import.meta.hot");
	expect(typeofGuardMatched).toBe(true);
	expect(webpackHotTypeofGuardMatched).toBe(true);
	expect(typeof import.meta.hot).toBe("object");
	expect(typeof import.meta.hot.accept).toBe("function");
	expect(typeof import.meta.webpackHot).toBe("object");

	const source = fs.readFileSync(__filename, "utf-8");
	const typeofKeyword = ["type", "of"].join("");
	const runtimeTypeof = `${typeofKeyword} module.hot !== "undefined"`;
	expect(source.split(runtimeTypeof)).toHaveLength(3);
});
