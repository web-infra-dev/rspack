const fs = require("fs");
const path = require("path");

it("should hand a real Chunk instance to the filename function and detect the entry chunk", async () => {
  // The entry chunk was named via `entry-[name].js`, proving the filename
  // function received a real Chunk instance and detected it as the entry chunk.
  expect(fs.existsSync(path.resolve(__dirname, "entry-main.js"))).toBe(true);

  // The dynamically imported chunk is not an entry chunk, so it was named via
  // the `async-[name].js` branch.
  const mod = await import(/* webpackChunkName: "lazy" */ "./lazy");
  expect(mod.default).toBe(42);
  expect(fs.existsSync(path.resolve(__dirname, "async-lazy.js"))).toBe(true);
});
