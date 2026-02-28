it("should have correct value", () => {
  if (WATCH_STEP === "0") {
    expect(require("./entry.txt")).toBe("entry for dep0");
  } else if (WATCH_STEP === "1") {
    expect(() => {
      require("./entry.txt")
    }).toThrow();
  } else if (WATCH_STEP === "2") {
    expect(require("./entry.txt")).toBe("entry for dep2");
  }
});
