it("should generate correct sri hash for css chunk", async () => {
  await import("./chunk");
  const links = Array.from(document.querySelectorAll("link"));
  expect(links.length).toBe(2);
  expect(links[0].integrity).toContain("sha256-");
  expect(links[0].integrity).toContain("sha384-");
  expect(links[1].integrity).toContain("sha256-");
  expect(links[1].integrity).toContain("sha384-");
});
