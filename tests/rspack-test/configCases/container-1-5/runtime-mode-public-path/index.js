it("uses rspack runtime public path in module federation runtime mode", async () => {
  expect(await import("shared-lib")).toBeTruthy();
});
