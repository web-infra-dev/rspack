require("./App");

it("should map to the original content if `module` enabled", async () => {
	const path = require("path");
	const fs = require("fs");
	const sourceMap = require("source-map");

	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const generated = fs.readFileSync(__filename, "utf-8");
	const app = fs.readFileSync(path.resolve(CONTEXT, "./App.jsx"), "utf-8");
	const map = JSON.parse(source);
	const consumer = await new sourceMap.SourceMapConsumer(map);
	const appSourceIndex = map.sources.indexOf("webpack:///./App.jsx")
	expect(appSourceIndex).toBeGreaterThanOrEqual(0);
	expect(map.sourcesContent[appSourceIndex]).toEqual(app);
	const STUB = "Hello Rspack!";
	const { line, column } = consumer.originalPositionFor(
		positionFor(generated, STUB)
	);
	const { line: originalLine, column: originalColumn } = positionFor(app, STUB);
	expect(line).toBe(originalLine);
	expect(column).toBe(originalColumn);
});

const positionFor = (content, text) => {
	let lines = content.split(/\r?\n/);
	for (let i = 0; i < lines.length; i++) {
		const column = lines[i].indexOf(text);
		if (column >= 0) return { line: i + 1, column };
	}
	return null;
};
