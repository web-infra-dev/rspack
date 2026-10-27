it("should generate sri hash for a chunk that only contains extracted css", async () => {
  await import("./chunk");

  const link = document.querySelector("link");
  expect(link.integrity).toContain("sha384-");
});
