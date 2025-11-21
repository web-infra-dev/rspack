it("should be bundled in foo chunk", () => {
    expect(__filename).toContain("foo.js");
});
