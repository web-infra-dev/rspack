const scalarLib = require("scalar-lib");
const lib = require("lib");

it("should install scalar and ordered eager consumes before a synchronous entry", () => {
	expect(scalarLib).toBe("scalar-lib");
	expect(lib).toBe("lib");
});
