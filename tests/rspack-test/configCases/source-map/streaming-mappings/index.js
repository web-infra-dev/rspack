import { café, résumé } from "./mapped";

it("preserves execution through loader and minifier source maps", () => {
  expect(café("abc")).toBe("😀雪abc3");
  expect(résumé("abc")).toBe("😀雪ABC3");
});

it("preserves named mappings and UTF-16 columns after non-ASCII text", async () => {
  const fs = require("fs");
  const { SourceMapConsumer } = require("source-map");
  const map = JSON.parse(fs.readFileSync(__filename + ".map", "utf8"));
  const generatedLines = fs.readFileSync(__filename, "utf8").split("\n");
  const sourceIndex = map.sources.findIndex(source => source.endsWith("/mapped.js"));
  expect(sourceIndex).toBeGreaterThanOrEqual(0);
  const lines = map.sourcesContent[sourceIndex].split("\n");
  const expectedNames = new Set(["café", "résumé", "prefix", "suffix", "value"]);
  const seen = new Set();
  let checkedAfterSurrogatePair = false;
  const consumer = await new SourceMapConsumer(map);
  try {
    consumer.eachMapping(mapping => {
      if (!mapping.source?.endsWith("/mapped.js") || !expectedNames.has(mapping.name)) {
        return;
      }
      const line = lines[mapping.originalLine - 1];
      expect(line.slice(mapping.originalColumn, mapping.originalColumn + mapping.name.length))
        .toBe(mapping.name);
      const generatedLine = generatedLines[mapping.generatedLine - 1];
      expect(
        generatedLine.slice(mapping.generatedColumn, mapping.generatedColumn + mapping.name.length)
      ).toBe(mapping.name);
      seen.add(mapping.name);
      if (mapping.name === "suffix" && line.includes("😀")) {
        expect(mapping.originalColumn).toBe(line.indexOf("suffix"));
        checkedAfterSurrogatePair = true;
      }
    });
    expect([...seen].sort()).toEqual([...expectedNames].sort());
    expect(checkedAfterSurrogatePair).toBe(true);
  } finally {
    consumer.destroy();
  }
});
