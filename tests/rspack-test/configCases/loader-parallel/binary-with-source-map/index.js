const logo = require("./logo.png");

it("should compile binary file with source-map", () => {
	expect(logo.endsWith(".png")).toBe(true);
});
