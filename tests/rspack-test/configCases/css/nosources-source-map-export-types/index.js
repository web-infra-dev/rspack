import * as css from "./style.css";

globalThis.__keepCssAlive = css;

const fs = __nodeFs;
const path = __nodePath;
const NodeBuffer = __NodeBuffer;

const EXPORT_TYPES = ["link", "text", "style", "css-style-sheet"];
const exportType = EXPORT_TYPES[__STATS_I__];
const outputPath = __STATS__.children[__STATS_I__].outputPath;

const expectNoSourcesContent = (map) => {
	expect(map.version).toBe(3);
	expect(Array.isArray(map.sources)).toBe(true);
	expect(map.sources.length).toBeGreaterThan(0);
	// `nosources-source-map` must drop sourcesContent entirely (or leave
	// every entry null). It must not contain any of the original CSS body.
	if (map.sourcesContent !== undefined) {
		for (const content of map.sourcesContent) {
			expect(content).toBeNull();
		}
	}
};

const expectValidMap = (map) => {
	expect(map.version).toBe(3);
	expect(Array.isArray(map.sources)).toBe(true);
	expect(map.sources.length).toBeGreaterThan(0);
	expect(typeof map.mappings).toBe("string");
};

const SOURCE_MAPPING_DATA_URI =
	/sourceMappingURL=data:application\/json(?:;charset=[^;,]+)?;base64,([A-Za-z0-9+/=]+)/;

it(`should not embed sourcesContent for nosources-source-map (exportType="${exportType}")`, () => {
	if (exportType === "link") {
		const mapFile = path.resolve(outputPath, `bundle${__STATS_I__}.css.map`);
		expect(fs.existsSync(mapFile)).toBe(true);
		const raw = fs.readFileSync(mapFile, "utf-8");
		expectNoSourcesContent(JSON.parse(raw));
		// And nothing CSS-textual should leak into the .css.map file at all.
		expect(raw).not.toContain(".nosources-test-class");
		return;
	}

	// JS source map should be emitted and parseable for JS-backed export types.
	const jsMapFile = path.resolve(outputPath, `bundle${__STATS_I__}.js.map`);
	expect(fs.existsSync(jsMapFile)).toBe(true);
	const jsMapRaw = fs.readFileSync(jsMapFile, "utf-8");
	expectValidMap(JSON.parse(jsMapRaw));

	// The inline data URI map embedded in the CSS string should also be
	// parseable for text/style/css-style-sheet export types.
	const bundle = fs.readFileSync(
		path.resolve(outputPath, `bundle${__STATS_I__}.js`),
		"utf-8"
	);
	const match = bundle.match(SOURCE_MAPPING_DATA_URI);
	expect(match).not.toBeNull();
	const decoded = NodeBuffer.from(match[1], "base64").toString("utf-8");
	expectValidMap(JSON.parse(decoded));
});
