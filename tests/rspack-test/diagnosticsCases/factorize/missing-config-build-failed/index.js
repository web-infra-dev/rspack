it("should throw JavaScript parsing error if a module's module type is not defined", () => {
  let errored = false
  try {
	  require("./font.ttf");
  } catch(err) {
    errored = true
  }
  expect(errored).toBeTruthy()
})