import { join } from "./app";

it("should match non-object layer branches as external items", () => {
  expect(typeof join).toBe("function");
  expect(join("a", "b")).toBe(process.platform === "win32" ? "a\\b" : "a/b");
});
