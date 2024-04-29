const { pathToFileURL } = require("url");
const url = pathToFileURL(
	require("path").resolve("./tests/normalCases/esm/import-meta-property/index.js")
).toString();
it("import.meta.url.xxx", () => {
	expect(typeof import.meta.url.length).toBe("number");
	expect(import.meta.url.replace("import-meta-property", "xxx")).toBe(
		url.replace("import-meta-property", "xxx")
	);
});
