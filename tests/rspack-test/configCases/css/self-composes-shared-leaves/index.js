import * as external from "./external.module.css";
import * as self from "./self.module.css";

// The external branches are copied from webpack's css/css-loader dedup fixture.
const expected = "top primaryButton button secondaryButton button";

it("should preserve shared transitive classes across external composition branches", () => {
  expect(external.top).toBe(expected);
});

it("should preserve shared transitive classes across same-module composition branches", () => {
  expect(self.primaryButton).toBe("primaryButton button");
  expect(self.secondaryButton).toBe("secondaryButton button");
  expect(self.top).toBe(expected);
});
