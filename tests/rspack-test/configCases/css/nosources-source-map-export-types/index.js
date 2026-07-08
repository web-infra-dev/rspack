import * as css from "./style.css";

globalThis.__keepCssAlive = css;

const fs = __nodeFs;
const path = __nodePath;
const NodeBuffer = __NodeBuffer;

const EXPORT_TYPES = ["link", "text", "style", "css-style-sheet"];
const exportType = EXPORT_TYPES[__STATS_I__];
const outputPath = __STATS__.children[__STATS_I__].outputPath;

const findOutputFile = (exactName, pattern) => {
  const exactFile = path.resolve(outputPath, exactName);
  if (fs.existsSync(exactFile)) {
    return exactFile;
  }
  const fallback = fs.readdirSync(outputPath).find((file) => pattern.test(file));
  expect(fallback).toBeDefined();
  return path.resolve(outputPath, fallback);
};

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

const SOURCE_MAPPING_DATA_URI =
  /sourceMappingURL=data:application\/json(?:;charset=[^;,]+)?;base64,([A-Za-z0-9+/=]+)/;

it(`should not embed sourcesContent for nosources-source-map (exportType="${exportType}")`, () => {
  if (exportType === "link") {
    const mapFile = findOutputFile(`bundle${__STATS_I__}.css.map`, /\.css\.map$/);
    const raw = fs.readFileSync(mapFile, "utf-8");
    expectNoSourcesContent(JSON.parse(raw));
    // And nothing CSS-textual should leak into the .css.map file at all.
    expect(raw).not.toContain(".nosources-test-class");
    return;
  }

  // JS source map should also strip sourcesContent.
  const jsMapFile = findOutputFile(`bundle${__STATS_I__}.js.map`, /\.js\.map$/);
  const jsMapRaw = fs.readFileSync(jsMapFile, "utf-8");
  expectNoSourcesContent(JSON.parse(jsMapRaw));
  // The CSS module's emitted JS wrapper would normally land in
  // sourcesContent (and that wrapper embeds the CSS body) — under
  // `nosources` it must be gone.
  expect(jsMapRaw).not.toContain(".nosources-test-class");

  // And — the actual regression — the inline data URI map embedded in
  // the CSS string must also have its sourcesContent stripped.
  const bundle = fs.readFileSync(
    findOutputFile(`bundle${__STATS_I__}.js`, /\.js$/),
    "utf-8"
  );
  const match = bundle.match(SOURCE_MAPPING_DATA_URI);
  expect(match).not.toBeNull();
  const decoded = NodeBuffer.from(match[1], "base64").toString("utf-8");
  expectNoSourcesContent(JSON.parse(decoded));
  expect(decoded).not.toContain(".nosources-test-class");
});
