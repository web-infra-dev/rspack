import { LongExportName } from "./barrel";

it("keeps the imported value working", () => {
	expect(LongExportName).toBe("value");
});

it("mangles a known export across empty star reexports", () => {
	const source = require("fs").readFileSync(__filename, "utf-8");

	expect(source).not.toMatch(/^\s+LongExportName: \(\) =>/m);
	expect(source).not.toMatch(
		/^\/\*! export LongExportName .*provision prevents renaming/m
	);
	expect(source).toMatch(/^\s+a: \(\) =>/m);
});
