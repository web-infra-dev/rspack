it("should unwrap dynamic source-phase wasm module in modern-module output", async () => {
  const { createAdd, loadAdd } = await import(/* rspackIgnore: true */ "./index.mjs");
  const add = await createAdd();
  const evaluatedAdd = await loadAdd();

  expect(add(1, 2)).toBe(3);
  expect(evaluatedAdd(2, 3)).toBe(5);
});
