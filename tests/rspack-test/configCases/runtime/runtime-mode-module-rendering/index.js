export { value } from "./lib";

import { value } from "./lib";

it("keeps the exported binding live", () => {
  expect(value).toBe(42);
});
