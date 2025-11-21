it("should generate correct sri hash for css extract chunk", async () => {
  await import("./style.module.css");
  const link = document.querySelector("link");
  expect(link.integrity).toContain("sha256-");
  expect(link.integrity).toContain("sha384-");
});
