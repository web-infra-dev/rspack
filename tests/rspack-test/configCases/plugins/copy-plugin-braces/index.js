const fs = require("fs");
const path = require("path");

const read = file =>
	fs.readFileSync(path.join(__STATS__.outputPath, file), "utf-8").trim();
const exists = file => fs.existsSync(path.join(__STATS__.outputPath, file));

it("should copy files when a brace pattern is combined with a wildcard", () => {
	expect(read("wildcard/foo.en.yml")).toBe("from foo");
	expect(read("wildcard/bar.de.yml")).toBe("from bar");
	expect(exists("wildcard/qux.en.yml")).toBe(false);
});

it("should copy files when a brace is the only glob character", () => {
	expect(read("literal/alpha.txt")).toBe("from alpha");
	expect(read("literal/beta.txt")).toBe("from beta");
	expect(exists("literal/gamma.txt")).toBe(false);
});

it("should copy files when a brace pattern matches a directory segment", () => {
	expect(read("nested/src/one/deep/x.txt")).toBe("from one");
	expect(read("nested/src/two/y.txt")).toBe("from two");
	expect(exists("nested/src/three/z.txt")).toBe(false);
});
