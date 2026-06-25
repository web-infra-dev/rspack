import { used } from "./common";

it("uses commonjs exports in rspack runtime mode", () => {
  expect(used).toBe("used");
});
