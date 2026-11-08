it("should unwrap dynamic source-phase wasm module in modern-module output", async () => {
  const { createAdd } = await import(/* rspackIgnore: true */ "./index.mjs");
  const add = await createAdd();

  expect(add(1, 2)).toBe(3);
});
