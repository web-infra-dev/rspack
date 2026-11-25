const fs = require("fs");
const path = require("path");

it("should copy files and preserve attributes from the JavaScript input file system", () => {
	const outputPath = __STATS__.outputPath;
	const direct = path.join(outputPath, "copied/direct.txt");
	const stat = fs.statSync(direct);
	expect(stat.atimeMs).toBe(1_600_000_000_000);
	expect(stat.mtimeMs).toBe(1_500_000_000_000);
	if (process.platform !== "win32") expect(stat.mode & 0o777).toBe(0o751);

	expect(fs.readFileSync(direct, "utf-8")).toBe("direct from js input fs");
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
