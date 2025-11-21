import a from "/dir/a";

it("should support roots", async () => {
  expect(a).toBe("a");
  const b = "b"; // keep /dir2/ as a context module
  const m = await import("/dir2/" + b);
  expect(m.default).toBe("b")
})
