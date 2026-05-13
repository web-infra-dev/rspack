const fs = require("fs");
const path = require("path");

it("should copy files when from paths contain brackets", () => {
	const outputPath = __STATS__.outputPath;

	expect(
		fs
			.readFileSync(path.join(outputPath, "from-directory/file[1].txt"), "utf-8")
			.trim()
	).toBe("from directory");
	expect(
		fs.readFileSync(path.join(outputPath, "file[1].txt"), "utf-8").trim()
	).toBe("from file");
	expect(
		fs
			.readFileSync(path.join(outputPath, "from-object/file[1].txt"), "utf-8")
			.trim()
	).toBe("from file");
	expect(
		fs
			.readFileSync(
				path.join(outputPath, "from-dotfile/src/dotfiles/.env"),
				"utf-8"
			)
			.trim()
	).toBe("from case insensitive dotfile");
});
