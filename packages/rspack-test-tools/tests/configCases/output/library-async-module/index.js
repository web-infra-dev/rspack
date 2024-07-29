import { handler } from "./lambda.js"
it("handle should be a function", async function () {
  expect(handler()).toBe(1);
});
