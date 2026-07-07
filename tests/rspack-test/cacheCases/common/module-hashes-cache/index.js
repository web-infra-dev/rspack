import value from "./value";

it("should hit module hashes persistent cache on cold restart", async () => {
  expect(value).toBe(42);

  if (COMPILER_INDEX === 0) {
    await NEXT_START();
  }
});
