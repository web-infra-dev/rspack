require("./list.test");

it("should not include sourcesContent if noSources option is used", function() {
	var fs = require("fs");
	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var map = JSON.parse(source);
	expect(map).not.toHaveProperty("sourcesContent");
});

it("should preserve test call original positions when noSources option is used", async function() {
	var fs = require("fs");
	var path = require("path");
	var sourceMap = require("source-map");

	var source = fs.readFileSync(__filename + ".map", "utf-8");
	var generated = fs.readFileSync(__filename, "utf-8");
	var testSource = fs.readFileSync(path.resolve(CONTEXT, "./list.test.ts"), "utf-8");
	var map = JSON.parse(source);
	var consumer = await new sourceMap.SourceMapConsumer(map);
	var testSourceIndex = map.sources.indexOf("webpack:///./list.test.ts");
	expect(testSourceIndex).toBeGreaterThanOrEqual(0);
	expect(map).not.toHaveProperty("sourcesContent");

	var cases = [
		["test a", 3, 9],
		["test a-1", 4, 5],
		["test a-2", 9, 3],
	];
	for (var i = 0; i < cases.length; i++) {
		var c = cases[i];
		var actual = positionsFor(generated, c[0])
			.map(function(position) {
				return consumer.originalPositionFor(position);
			})
			.find(function(position) {
				return /\/list\.test\.ts$/.test(position.source || "");
			});
		var expected = callSitePositionFor(testSource, c[0]);
		expect(expected).toEqual({ line: c[1], column: c[2] });
		expect(actual).toBeTruthy();
		expect(actual.line).toBe(expected.line);
		expect(actual.column).toBe(expected.column);
	}
});

if (Math.random() < 0) require("./test.js");

var positionFor = function(content, text) {
	var lines = content.split(/\r?\n/);
	for (var i = 0; i < lines.length; i++) {
		var column = lines[i].indexOf(text);
		if (column >= 0) return { line: i + 1, column };
	}
	return null;
};

var callSitePositionFor = function(content, text) {
	var position = positionFor(content, text);
	return { line: position.line, column: position.column - 1 };
};

var positionsFor = function(content, text) {
	var positions = [];
	var lines = content.split(/\r?\n/);
	for (var i = 0; i < lines.length; i++) {
		var column = lines[i].indexOf(text);
		while (column >= 0) {
			positions.push({ line: i + 1, column });
			column = lines[i].indexOf(text, column + text.length);
		}
	}
	return positions;
};
