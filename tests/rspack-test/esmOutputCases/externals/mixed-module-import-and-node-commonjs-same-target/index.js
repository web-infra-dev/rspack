import { sourcesNs } from "./namespace";
import { RawSource, SourceMapSource } from "./named";

export { sourcesNs };
export { RawSource, SourceMapSource };

it("should not emit duplicate identifiers when the same external target is rendered via module-import and node-commonjs", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.sourcesNs.RawSource).toBe(RawSource);
  expect(mod.sourcesNs.SourceMapSource).toBe(SourceMapSource);
  expect(mod.sourcesNs.RawSource).toBe(sourcesNs.RawSource);
  expect(mod.sourcesNs.SourceMapSource).toBe(sourcesNs.SourceMapSource);
});
