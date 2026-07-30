import { value, load, sync } from "./route";

it("should preserve sync and async outgoings across rebuilds", async () => {
  expect(value).toBe(
    WATCH_STEP === "0" ? "before:stable" : "after:stable",
  );
  expect(sync).toBe("first:second");
  expect(globalThis.__codesplit_side_effect__).toBe(true);
  expect(globalThis.__codesplit_order__).toEqual([
    "side-effect",
    "first",
    "second",
  ]);

  const modules = await load();
  const expected =
    WATCH_STEP === "5"
      ? ["inserted", "retargeted", true]
      : [
          WATCH_STEP === "0" || WATCH_STEP === "1" || WATCH_STEP === "2"
            ? true
            : "retargeted",
          true,
        ];
  expect(modules.map(module => module.lazy)).toEqual(expected);

  const expectedChunk =
    WATCH_STEP === "0" || WATCH_STEP === "1"
      ? "lazy.bundle.js"
      : "next.bundle.js";
  expect(__STATS__.assets.map(asset => asset.name)).toContain(expectedChunk);
  if (WATCH_STEP === "5") {
    expect(__STATS__.assets.map(asset => asset.name)).toContain(
      "inserted.bundle.js",
    );
  }

  if (WATCH_STEP === "4") {
    const movedOrigin = __STATS__.chunks
      .flatMap(chunk => chunk.origins || [])
      .find(origin => origin.request === "./other");
    expect(movedOrigin).toBeDefined();
    const startLine =
      typeof movedOrigin.loc === "string"
        ? Number.parseInt(movedOrigin.loc, 10)
        : movedOrigin.loc.start.line;
    expect(startLine).toBe(11);
  }
});
