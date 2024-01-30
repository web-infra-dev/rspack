it("should not parse require under esm", () => {
  let count = 0;
  try {
    require('./in-exists');
  } catch (_err) {
    count += 1;
  }
  expect(count).toBe(1);
})