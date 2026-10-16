import test from "./index.module.css";
import { res } from "./lib.js";

it("should not mangle css module", () => {
  res;
  // Using this to trigger a none provided export
  test.res;

  expect(Object.keys(test)).toEqual(["test"]);
  expect(test.test).toMatchFileSnapshotSync(`${__SNAPSHOT__}/test.txt`);
});
