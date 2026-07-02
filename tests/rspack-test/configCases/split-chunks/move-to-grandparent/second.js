it("should handle indirect children with multiple parents correctly", function() {
  return import("./pageB").then(b => {
    expect(b.default).toBe("reuse");
  });
});
