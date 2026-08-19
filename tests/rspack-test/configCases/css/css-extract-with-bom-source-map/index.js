require("./plain.css");

const fs = require("fs");
const path = require("path");

const read = (name) =>
	fs.readFileSync(path.resolve(__dirname, name), "utf-8");

it("should strip the BOM without shifting source map columns", () => {
	expect(read("bom.css")).not.toContain("\uFEFF");

	// bom.css and main.css hold identical CSS, so stripping the BOM must leave
	// the mappings identical too. Dropping it from the content alone would shift
	// every generated column on the first line left by one.
	const withBom = JSON.parse(read("bom.css.map"));
	const withoutBom = JSON.parse(read("main.css.map"));
	expect(withBom.mappings).toBe(withoutBom.mappings);
});
