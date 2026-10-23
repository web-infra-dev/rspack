import child from "./data.txt?child";
import parent from "./data.txt?parent";
import oneOf from "./data.txt?one-of";
import oneOfParent from "./data.txt?one-of-parent";
import suppressed from "./data.txt?suppressed";
import crossLevel from "./data.txt?cross-level";

it("should compare child priorities with parent and sibling priorities", () => {
  expect(child).toBe("priority fixture\n");
  expect(parent).toBe("priority fixture\n");
  expect(oneOfParent).toBe("priority fixture\n");
  expect(crossLevel).toBe("priority fixture\n");
});

it("should preserve first-match behavior in oneOf regardless of priority", () => {
  expect(oneOf).toBe("priority fixture\n");
  expect(suppressed).toBe("priority fixture\n");
});
