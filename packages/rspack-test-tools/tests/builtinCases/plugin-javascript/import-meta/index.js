const { pathToFileURL } = require("url");
const url = pathToFileURL(
	"./tests/builtinCases/plugin-javascript/import-meta/index.js",
).toString();

const filename = __filename;
const dirname = __dirname;

it('typeof import.meta === "object"', () => {
	expect(typeof import.meta).toBe("object");
	if (typeof import.meta !== "object") require("fail");
});

it('typeof import.meta.url === "string"', () => {
	expect(typeof import.meta.url).toBe("string");
	if (typeof import.meta.url !== "string") require("fail");
});

it('typeof import.meta.filename === "string"', () => {
	expect(typeof import.meta.filename).toBe("string");
	if (typeof import.meta.filename !== "string") require("fail");
});

it('typeof import.meta.dirname === "string"', () => {
	expect(typeof import.meta.dirname).toBe("string");
	if (typeof import.meta.dirname !== "string") require("fail");
});

// [TODO]: import.meta.url behaves differently on Windows - needs investigation and fix
// it("should return correct import.meta.url", () => {
// 	expect(import.meta.url).toBe(url);
// 	expect(import.meta["url"]).toBe(url);
// 	expect("my" + import.meta.url).toBe("my" + url);
// 	if (import.meta.url.indexOf("index.js") === -1) require("fail");
// });

it("should return correct import.meta.filename", () => {
	expect(import.meta.filename).toBe(filename);
	expect(import.meta["filename"]).toBe(filename);
	expect("my" + import.meta.filename).toBe("my" + filename);
	if (import.meta.filename.indexOf("index.js") === -1) require("fail");
});

it("should return correct import.meta.dirname", () => {
	expect(import.meta.dirname).toBe(dirname);
	expect(import.meta["dirname"]).toBe(dirname);
	expect("my" + import.meta.dirname).toBe("my" + dirname);
	if (import.meta.dirname.indexOf("import-meta") === -1) require("fail");
});

it("should return correct import.meta.resolve", () => {
	expect(typeof import.meta.resolve).toBe("function");
	if (typeof import.meta.resolve !== "function") require("fail");
	expect(import.meta.resolve("./index.js")).toBe(__filename);
	expect(import.meta["resolve"]("./index.js")).toBe(__filename);
});

it("should return undefined for unknown property", () => {
	expect(import.meta.other).toBe(undefined);
	if (typeof import.meta.other !== "undefined") require("fail");
	expect(() => import.meta.other.other.other).toThrow();
});

it("should add warning on direct import.meta usage", () => {
	expect(Object.keys(import.meta)).toHaveLength(0);
});
