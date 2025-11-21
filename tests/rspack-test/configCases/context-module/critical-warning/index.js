it("should import context module", async () => {
  const n = "a"
  const { default: a1 } = await import(`./sub/${n}`);
  expect(a1).toBe("a")
  const { default: a2 } = await import("./sub/".concat(n));
  expect(a2).toBe("a")
  const n2 = "./sub2/a.js";
  try {
    await import(n2);
  } catch (e) {
    expect(e.code).toBe("MODULE_NOT_FOUND");
  }
})