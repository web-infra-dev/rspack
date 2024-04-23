const log = jest.spyOn(console, "log").mockImplementation(() => {});
const error = jest.spyOn(console, "error").mockImplementation(() => {});

console.log("Hello, world!");
console.error("Hello, world!");

it("@swc/plugin-remove-console should remove console.log", () => {
  expect(log).toHaveBeenCalledTimes(0);
  expect(error).toHaveBeenCalledTimes(1);
  log.mockReset();
  error.mockReset();
});
