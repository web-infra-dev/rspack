const { pathToFileURL } = require("url");
const url = pathToFileURL(
	require("path").resolve("./normalCases/esm/import-meta-property/index.js")
).toString();
it("import.meta.url.xxx", () => {
	expect(typeof import.meta.url.length).toBe("number");
	expect(import.meta.url.replace("import-meta-property", "xxx")).toBe(
		url.replace("import-meta-property", "xxx")
	);
});

it('import.meta?.env?.X', () => {
	expect(import.meta?.env?.X).toBeUndefined();
	expect(typeof import.meta?.env?.X).toBe("undefined");
});

it('import.meta.env?.X', () => {
	expect(import.meta.env?.X).toBeUndefined();
	expect(typeof import.meta.env?.X).toBe("undefined");
});

it('import.meta.kkk.env?.X;', () => {
	expect(() => import.meta.kkk.env?.X).toThrow();
	expect(() => typeof import.meta.kkk.env?.X).toThrow();
});

it('import.meta.ttt?.env.X', () => {
	expect(import.meta.ttt?.env.X).toBeUndefined();
	expect(typeof import.meta.ttt?.env.X).toBe("undefined");
});