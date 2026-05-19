export * as out from "fs";

it("should export an external module namespace with export-star-as syntax", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(typeof mod.out.readFile).toBe("function");
  expect(typeof mod.out.readFileSync).toBe("function");
});
