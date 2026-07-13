import value from "./value";

it("should log module hashes affected modules after persistent cache recovery", async () => {
  expect(value).toBe(42);

  if (COMPILER_INDEX === 0) {
    await NEXT_START();
  }
});
