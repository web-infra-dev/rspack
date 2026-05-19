import value from "./esm-for-concate";

it("should mark concatenated modules as orphan in stats", () => {
  expect(value).toBe(42);

  const module = __STATS__.modules.find(m =>
    m.identifier
      .replaceAll("\\", "/")
      .includes("configCases/concatenate-modules/stats-orphan/esm-for-concate.js")
  );

  expect(module).toBeDefined();
  expect(module.orphan).toBe(true);
});
