import * as fsNs from "fs";
import { readFile, readFileSync } from "fs";

it("should keep namespace and named external imports aligned", () => {
  expect(fsNs.readFile).toBe(readFile);
  expect(fsNs.readFileSync).toBe(readFileSync);
});
