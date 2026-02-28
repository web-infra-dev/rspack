const promise = Promise.all([
  import("./a"),
  import("./b"),
  import("./c"),
]);

it("should not change the chunk id for updating a module", async () => {
  const [a, b, c] = await promise;
  expect(await a.default()).toBe(`a1|${WATCH_STEP}`);
  expect(await b.default()).toBe("b1");
  expect(c.default).toBe("c");
  const chunks = __STATS__.chunks.map(c => c.id);
  chunks.sort();
  expect(chunks).toEqual(['a1_js', 'a_js', 'b1_js', 'b_js', 'c_js', 'main']);
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
        expect(content).toContain("6 chunks are updated by set_chunk_id, with 1 chunks using name as id, and 0 unnamed chunks");
        break;
      case "1":
        expect(content).toContain("2 chunks are updated by set_chunk_id, with 1 chunks using name as id, and 0 unnamed chunks");
        break;
    }
  }
});
