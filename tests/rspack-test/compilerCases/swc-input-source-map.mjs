import { SourceMapConsumer, SourceMapGenerator } from "source-map";

function positionOf(code, token) {
	const prefix = code.slice(0, code.indexOf(token));
	const lines = prefix.split("\n");
	return { line: lines.length, column: lines.at(-1).length };
}

const cases = [
	{
		description: "preserve identifier names when the input source map has no names",
		source: "const user = value;\n",
		inputMap(source) {
			const map = new SourceMapGenerator();
			map.addMapping({
				generated: positionOf(source, "user"),
				original: { line: 1, column: 6 },
				source: "original.js"
			});
			map.setSourceContent("original.js", source);
			return map.toString();
		},
		check(code, map) {
			expect(map.originalPositionFor(positionOf(code, "user"))).toEqual({
				source: "original.js", line: 1, column: 6, name: "user"
			});
		}
	},
	{
		description: "keep gaps in the input source map unmapped",
		source: "const a = 1; const b = 2;\n",
		inputMap(source) {
			const map = new SourceMapGenerator();
			map.addMapping({
				generated: { line: 1, column: 0 },
				original: { line: 1, column: 0 },
				source: "original.js"
			});
			map.addMapping({ generated: positionOf(source, "b") });
			map.setSourceContent("original.js", source);
			return map.toString();
		},
		check(code, map) {
			expect(map.originalPositionFor(positionOf(code, "a")).source).toBe("original.js");
			expect(map.originalPositionFor(positionOf(code, "b"))).toEqual({
				source: null, line: null, column: null, name: null
			});
		}
	},
	{
		description: "preserve absolute input sources under sourceRoot",
		source: "const first = second;\n",
		inputMap(source) {
			const map = new SourceMapGenerator();
			for (const [token, file] of [
				["first", "/src/app.js"],
				["second", "https://example.com/vendor.js"]
			]) {
				map.addMapping({
					generated: positionOf(source, token),
					original: { line: 1, column: 0 },
					source: file
				});
				map.setSourceContent(file, source);
			}
			return JSON.stringify({ ...map.toJSON(), sourceRoot: "/root" });
		},
		check(code, map) {
			expect(map.sources).toEqual(["/src/app.js", "https://example.com/vendor.js"]);
			expect(map.originalPositionFor(positionOf(code, "first")).source).toBe("/src/app.js");
			expect(map.originalPositionFor(positionOf(code, "second")).source).toBe("https://example.com/vendor.js");
		}
	}
];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default cases.map(testCase => ({
	description: `should ${testCase.description} in swc.transform`,
	async check({ compiler }) {
		const result = await compiler.rspack.experiments.swc.transform(testCase.source, {
			filename: "input.js",
			sourceMaps: true,
			inputSourceMap: testCase.inputMap(testCase.source),
			jsc: { target: "es2022", parser: { syntax: "ecmascript" } }
		});
		await SourceMapConsumer.with(result.map, null, map => {
			testCase.check(result.code, map);
		});
	}
}));
