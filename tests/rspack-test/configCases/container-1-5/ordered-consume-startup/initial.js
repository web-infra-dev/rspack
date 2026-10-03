// A synchronous entry with an eager ordered consume: the factory must exist
// when this module runs, so startup has to await scope initialization.
const lib = require("lib");

it("should await ordered scope initialization before a synchronous entry", () => {
	expect(lib).toBe("lib");
});
