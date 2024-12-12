const promise = Promise.all([
  import("./a"),
  import("./a?1"),
  import("./a?2"),
]);

it("should not change the chunk id for updating a module", async () => {
  const [a, b, c] = await promise;
  expect(a.default).toBe(WATCH_STEP + "id");
  expect(b.default).toBe(WATCH_STEP + "id?1");
  expect(c.default).toBe(WATCH_STEP + "id?2");
  expect(__STATS__.chunks.map(c => c.id)).toEqual(['a_js', 'a_js_1', 'a_js_2', 'main']);
})

it("should have correct log when incremental enabled", async () => {
  const fs = require("fs/promises");
  const path = require("path");
  const statsString = await fs.readFile(path.resolve(__dirname, `stats.${WATCH_STEP}.txt`), 'utf-8');
  const incrementalLog = /LOG from rspack\.incremental\.chunkIds[\s\S]*?LOG/.exec(statsString);
  if (incrementalLog) {
    const content = incrementalLog[0];
    switch (WATCH_STEP) {
      case "0":
        expect(content.includes("4 chunks are affected, 4 in total")).toBe(true);
        expect(content.includes("4 chunks are updated by set_chunk_id, with 1 chunks using name as id, and 0 unnamed chunks")).toBe(true);
        break;
      case "1":
        expect(content.includes("0 chunks are affected, 4 in total")).toBe(true);
        expect(content.includes("0 chunks are updated by set_chunk_id, with 0 chunks using name as id, and 0 unnamed chunks")).toBe(true);
        break;
    }
  }
});
