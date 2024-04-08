const { pathToFileURL } = require("url");
const url = pathToFileURL(
	require("path").resolve("./tests/cases/esm/import-meta/index.js")
).toString();
const pkg = require("../../../../package.json");
const webpackVersion = parseInt(
	pkg.webpackVersion,
	10
);

const rspackVersion = parseInt(
	pkg.version,
	10
);
console.log('---------------------------------')
console.log(rspackVersion, import.meta.rspack);
it('typeof import.meta === "object"', () => {
	expect(typeof import.meta).toBe("object");
	// if (typeof import.meta !== "object") require("fail");
});

it('typeof import.meta.url === "string"', () => {
	expect(typeof import.meta.url).toBe("string");
	// if (typeof import.meta.url !== "string") require("fail");
});

it("should return correct import.meta.url", () => {
	expect(import.meta.url).toBe(url);
	expect(import.meta["url"]).toBe(url);
	expect("my" + import.meta.url).toBe("my" + url);
	expect(import.meta.url.indexOf("index.js") === -1).toBe(false);
	// if (import.meta.url.indexOf("index.js") === -1) require("fail");
});

it('typeof import.meta.webpack === "number"', () => {
	expect(typeof import.meta.webpack).toBe("number");
	// if (typeof import.meta.webpack !== "number") require("fail");
});

it("should return correct import.meta.webpack", () => {
	expect(import.meta.webpack).toBe(webpackVersion);
	// if (import.meta.webpack < 5) require("fail");
	// if (import.meta.webpack >= 5) {
	// } else {
	// 	require("fail");
	// }
});

it('typeof import.meta.rspack === "number"', () => {
	expect(typeof import.meta.rspack).toBe("number");
	// if (typeof import.meta.webpack !== "number") require("fail");
});

it("should return correct import.meta.rspack", () => {
	expect(import.meta.rspack).toBe(rspackVersion);
	// if (import.meta.webpack < 5) require("fail");
	// if (import.meta.webpack >= 5) {
	// } else {
	// 	require("fail");
	// }
});

it("should return undefined for unknown property", () => {
	expect(import.meta.other).toBe(undefined);
	// if (typeof import.meta.other !== "undefined") require("fail");
	// expect(() => import.meta.other.other.other).toThrow();
});

it("should add warning on direct import.meta usage", () => {
	// expect(Object.keys(import.meta)).toHaveLength(0);
});

// it("should support destructuring assignment", () => {
// 	let version, url2, c;
// 	({ webpack: version } = { url: url2 } = { c } = import.meta);
// 	expect(version).toBeTypeOf("number");
// 	expect(url2).toBe(url);
// 	expect(c).toBe(undefined);
// });

it('import.meta.env and import.meta.env.xxx should return false', () => {
	expect(import.meta.env).toBe(false);
	expect(import.meta.env.xxx).toBe(false);
});
