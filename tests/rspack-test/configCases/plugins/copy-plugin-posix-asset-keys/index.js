const fs = require("node:fs");
const path = require("node:path");

it("should emit copied assets with POSIX keys", () => {
	const assetNames = __STATS__.assets.map(({ name }) => name);
	expect(assetNames).toEqual(
		expect.arrayContaining([
			"copied/assets/glob/nested/one.txt",
			"template/simple.txt",
			"template/deep/two.txt"
		])
	);
	for (const name of assetNames) {
		expect(name).not.toContain("\\");
	}

	expect(
		fs
			.readFileSync(
				path.join(
					__STATS__.outputPath,
					"copied/assets/glob/nested/one.txt"
				),
				"utf-8"
			)
			.trim()
	).toBe("ordinary");
	expect(
		fs
			.readFileSync(
				path.join(__STATS__.outputPath, "template/simple.txt"),
				"utf-8"
			)
			.trim()
	).toBe("simple-template");
	expect(
		fs
			.readFileSync(
				path.join(__STATS__.outputPath, "template/deep/two.txt"),
				"utf-8"
			)
			.trim()
	).toBe("template");
});
