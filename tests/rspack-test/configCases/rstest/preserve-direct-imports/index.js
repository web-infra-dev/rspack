it("preserves direct imports without changing other consumers or indirect imports", async () => {
  globalThis.__directImportTrace = [];
  const consumer = await import("./consumer.js");
  expect(consumer.read()).toBe(42);
  expect(globalThis.__directImportTrace).toEqual([]);
  const protectedModule = await import("./protected.js");
  expect(protectedModule.read()).toBe(42);
  expect(globalThis.__directImportTrace.sort()).toEqual(["barrel", "init"]);
  expect((await import("./virtual-importer.js")).value).toBe(1);
  expect(globalThis.__directImportTrace).toContain("virtual");
});
