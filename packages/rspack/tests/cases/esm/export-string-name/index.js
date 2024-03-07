import { "string name" as stringNameExport, normalExport } from "./a";
import * as namespace from "./a";

it("should work well when export named with string name", function () {
  expect(normalExport).toBe("normal");
  expect(stringNameExport).toBe("string");
});


it("should work well when export namespace with string name", function () {
  expect(namespace.normalExport).toBe("normal");
  expect(namespace["string name"]).toBe("string");
});
