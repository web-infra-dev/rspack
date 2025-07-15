import imported from "./imported.mjs";
import { metaUrl } from "./meta";
import value from "./module";

const localMetaUrl = import.meta.url;

it("should allow to use externals in concatenated modules", () => {
	expect(imported).toBe(42);
	expect(value).toBe(40);
});

it("all bundled files should have same url, when parser.javascript.importMeta === false", () => {
	expect(localMetaUrl).toBe(metaUrl)
});
