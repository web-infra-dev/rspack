const stringFirst = require("./input.txt?string-first");
const bufferFirst = require("./input.txt?buffer-first");

it("should distinguish equal bytes with different content types in the loader cache", () => {
  const step = +WATCH_STEP;
  expect(stringFirst).toBe(step < 2 ? "\uFEFFhello" : "hello");
  expect(bufferFirst).toBe(step < 2 ? "hello" : "\uFEFFhello");
});
