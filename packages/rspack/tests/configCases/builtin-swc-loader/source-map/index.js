require("./a");

it("should generate correct sourceMap", async () => {
	const path = require("path");
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	const sourceContent = fs.readFileSync(
		path.resolve(__dirname, "../a.ts"),
		"utf-8"
	);
	expect(map.sources).toContain("./a.ts");
	expect(map.sourcesContent[1]).toEqual(sourceContent);

	checkStub("fo" + "o", sourceContent);
	checkStub("ba" + "r", sourceContent);
	checkStub("ba" + "z", sourceContent);
	checkStub(wrap("f" + 1), sourceContent);
	checkStub(wrap("b" + 1), sourceContent);
	checkStub(wrap("b" + 2), sourceContent);
	checkStub(wrap("ab" + "c"), sourceContent);
});

const wrap = v => `"${v}"`;
const checkStub = async (stub, sourceContent) => {
	const fs = require("fs");
	const { SourceMapConsumer } = require("source-map");

	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	const consumer = await new SourceMapConsumer(map);
	const generated = fs.readFileSync(__filename, "utf-8");
	const { line, column } = consumer.originalPositionFor(
		positionFor(generated, stub)
	);
	const { line: originalLine, column: originalColumn } = positionFor(
		sourceContent,
		stub
	);
	expect(line).toBe(originalLine);
	expect(column).toBe(originalColumn);
};

const positionFor = (content, text) => {
	let lines = content.split(/\r?\n/);
	for (let i = 0; i < lines.length; i++) {
		const column = lines[i].indexOf(text);
		if (column >= 0) return { line: i + 1, column };
	}
	return null;
};
