const stringFirst = require("./input.txt?string-first");
const bufferFirst = require("./input.txt?buffer-first");

it("should distinguish equal bytes with different content types in the loader cache", () => {
  const step = +WATCH_STEP;
  const runs = LOADER_CACHE_ENABLED ? (step < 2 ? 1 : 2) : step + 1;
  expect(stringFirst).toEqual({
    content: step < 2 ? "\uFEFFhello" : "hello",
    runs,
  });
  expect(bufferFirst).toEqual({
    content: step < 2 ? "hello" : "\uFEFFhello",
    runs,
  });
});
