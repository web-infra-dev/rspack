it("should throw if a module is failed to build", () => {
  let errored = false
  try {
    require("./logo.svg");
  } catch(err) {
    errored = true
    expect(err.message).toContain("Failed to load")
  }
  expect(errored).toBeTruthy()
});
