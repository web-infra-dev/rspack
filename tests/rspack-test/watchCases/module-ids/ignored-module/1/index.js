it("should have correct ignored value", () => {
  const ignored = require("ignored");
  expect(ignored).toEqual({});
});

it("should have correct log when incremental enabled", async () => {
  const fs = require("fs/promises");
  const path = require("path");
  const statsString = await fs.readFile(path.resolve(__dirname, `stats.${WATCH_STEP}.txt`), 'utf-8');
  const incrementalLog = /LOG from rspack\.incremental\.moduleIds[\s\S]*?LOG/.exec(statsString);
  if (incrementalLog) {
    const content = incrementalLog[0];
    expect(content.includes("with 0 unnamed modules")).toBe(true);
  }
});
