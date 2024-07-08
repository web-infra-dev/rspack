require("./a");

it("should generate correct sourceMap", async () => {
	const path = require("path");
	const fs = require("fs");
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	const sourceContent = fs.readFileSync(
		__dirname + "/" + require("!!./a.ts?resource"),
		"utf-8"
	);
	const aSourceIndex = map.sources.indexOf("webpack:///./a.ts");
	expect(aSourceIndex).toBeGreaterThanOrEqual(0);
	expect(map.sourcesContent[aSourceIndex]).toEqual(sourceContent);

	checkStub("foo", sourceContent);
	checkStub("bar", sourceContent);
	checkStub("baz", sourceContent);
	checkStub(wrap("f1"), sourceContent, false);
	checkStub(wrap("b1"), sourceContent, false);
	checkStub(wrap("b2"), sourceContent, false);
	checkStub(wrap("abc"), sourceContent, false);
});

const wrap = v => `"${v}"`;
const checkStub = async (stub, sourceContent, ident = true) => {
	const fs = require("fs");
	const { SourceMapConsumer } = require("source-map");

	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	const consumer = await new SourceMapConsumer(map);
	const generated = fs.readFileSync(__filename, "utf-8");
	const { line, column, name } = consumer.originalPositionFor(
		positionFor(generated, stub)
	);
	const { line: originalLine, column: originalColumn } = positionFor(
		sourceContent,
		stub
	);
	expect(line).toBe(originalLine);
	expect(column).toBe(originalColumn);
	if (ident) {
		expect(name).toBe(stub)
	}
};

const positionFor = (content, text) => {
	let lines = content.split(/\r?\n/);
	for (let i = 0; i < lines.length; i++) {
		const column = lines[i].indexOf(text);
		if (column >= 0) return { line: i + 1, column };
	}
	return null;
};
