import * as css from "./style.css";

globalThis.__keepCssAlive = css;

const fs = __nodeFs;
const path = __nodePath;

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

  // JS source map should also strip sourcesContent.
  const jsMapFile = path.resolve(outputPath, `bundle${__STATS_I__}.js.map`);
  expect(fs.existsSync(jsMapFile)).toBe(true);
  const jsMapRaw = fs.readFileSync(jsMapFile, "utf-8");
  expectNoSourcesContent(JSON.parse(jsMapRaw));
  // The CSS module's emitted JS wrapper would normally land in
  // sourcesContent (and that wrapper embeds the CSS body) — under
  // `nosources` it must be gone.
  expect(jsMapRaw).not.toContain(".nosources-test-class");

  // External maps must not be duplicated inside the embedded CSS text.
  const bundle = fs.readFileSync(
    path.resolve(outputPath, `bundle${__STATS_I__}.js`),
    "utf-8"
  );
  expect(bundle).not.toMatch(/\/\*# sourceMappingURL=data:/);
});
