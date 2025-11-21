it("nullish coalescing: undefined", function () {
  expect(undefined ?? "foo").toBe("foo");
});

it("nullish coalescing: false", function () {
  expect(false ?? "foo").toBe(false);
});