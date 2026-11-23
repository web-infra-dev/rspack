import * as css from "STYLE_UNDER_TEST";

globalThis.__keepCssAlive = css;

const devtools = [
  false,
  "source-map",
  "hidden-source-map",
  "nosources-source-map",
  "hidden-nosources-source-map",
  "inline-source-map",
  "inline-nosources-source-map"
];
const devtool = devtools[Math.floor(__STATS_I__ / 6)];

it(`should respect devtool=${devtool} in embedded CSS`, () => {
  const outputPath = __STATS__.children[__STATS_I__].outputPath;
  const bundle = __nodeFs.readFileSync(
    __nodePath.join(outputPath, `bundle${__STATS_I__}.js`),
    "utf-8"
  );
  // Match CSS comments, excluding the JavaScript bundle's own inline map.
  const match = bundle.match(
    /\/\*# sourceMappingURL=data:application\/json(?:;charset=[^;,]+)?;base64,([A-Za-z0-9+/=]+)/
  );
  if (devtool && devtool.startsWith("inline-")) {
    expect(match).not.toBeNull();
    const map = JSON.parse(
      __NodeBuffer.from(match[1], "base64").toString("utf-8")
    );
    const sourceFile = __STATS_I__ % 2 ? "style.less" : "style.css";
    expect(map.sources.some(source => source.includes(sourceFile))).toBe(true);
    if (devtool.includes("nosources")) {
      expect(map.sourcesContent || []).toHaveLength(0);
    } else {
      expect(
        map.sourcesContent.some(source => source.includes("source-map-test-class"))
      ).toBe(true);
    }
  } else {
    expect(match).toBeNull();
  }
});
