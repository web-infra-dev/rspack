const promise = Promise.all([
  import("./id-equals-name"),
  import(/* webpackChunkName: "id-equals-name_js" */ "./id-equals-name?1"),
]);

it("should have correct chunk id", async () => {
  const [a, b] = await promise;
  expect(a.default).toBe("id");
  expect(b.default).toBe("id?1");
  expect(__STATS__.chunks.map(c => c.id)).toEqual(['id-equals-name_js', 'id-equals-name_js0', 'main']);
})

it("should have correct log when incremental enabled", async () => {
  const fs = require("fs/promises");
  const path = require("path");
  const statsString = await fs.readFile(path.resolve(__dirname, `stats.${WATCH_STEP}.txt`), 'utf-8');
  const incrementalLog = /LOG from rspack\.incremental\.chunkIds[\s\S]*?LOG/.exec(statsString);
  if (incrementalLog) {
    const content = incrementalLog[0];
    expect(content.includes("3 chunks are updated by set_chunk_id, with 2 chunks using name as id, and 0 unnamed chunks")).toBe(true);
  }
});
