import fsPromises from "fs-promises";
import posix from "path-posix";

it("should handle array-type module externals with property access in ESM library output", () => {
	const fs = require("fs");
	const path = require("path");
	expect(fsPromises).toBe(fs.promises);
	expect(posix).toBe(path.posix);
});

export { fsPromises, posix };
