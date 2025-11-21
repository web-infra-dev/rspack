it("should throw if a module is failed to build", () => {
  let errored = false
  try {
    require("./syntax-error.json");
  } catch(err) {
    errored = true
    expect(err.message).toContain("Unexpected character }")
  }
  expect(errored).toBeTruthy()
});
