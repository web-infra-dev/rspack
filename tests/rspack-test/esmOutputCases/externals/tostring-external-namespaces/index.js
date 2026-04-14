import * as fsNs from "fs";
import * as pathNs from "path";

export default {
  fsTag: fsNs[Symbol.toStringTag],
  fsText: Object.prototype.toString.call(fsNs),
  pathTag: pathNs[Symbol.toStringTag],
  pathText: Object.prototype.toString.call(pathNs),
  basename: pathNs.basename("a/b.txt"),
};

it("should preserve module toStringTag on external namespace imports", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toEqual({
    fsTag: "Module",
    fsText: "[object Module]",
    pathTag: "Module",
    pathText: "[object Module]",
    basename: "b.txt",
  });
});
