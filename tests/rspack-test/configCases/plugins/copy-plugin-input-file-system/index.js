const fs = require("fs");
const path = require("path");

it("should copy files from JavaScript input file system", () => {
	const outputPath = __STATS__.outputPath;

	expect(
		fs.readFileSync(path.join(outputPath, "copied/direct.txt"), "utf-8")
	).toBe("direct from js input fs");
	expect(
		fs
			.readFileSync(
				path.join(outputPath, "copied/glob/virtual/nested/file.txt"),
				"utf-8"
			)
			.trim()
	).toBe("nested from js input fs");
	expect(
		fs
			.readFileSync(path.join(outputPath, "copied/glob/virtual/.env"), "utf-8")
			.trim()
	).toBe("dotfile from js input fs");
});
