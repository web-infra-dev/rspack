import { read } from "./importer.js";
import { trace } from "./state.js";

it("only overrides targets declared side-effect-free", () => {
  expect(read()).toBe(42);
  expect(trace.sort()).toEqual(
    (PRESERVE
      ? ["true", "undeclared", "false", "rule", "array-kept", "array-omitted", "package"]
      : ["true", "undeclared", "array-kept"]
    ).sort(),
  );
});
