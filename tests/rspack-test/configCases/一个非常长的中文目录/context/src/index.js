const message = require("./message");

it("should compile entry when context contains multi-byte characters", () => {
  expect(message).toBe("ok");
});
