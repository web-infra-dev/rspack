import layerMatch from './layer-match';
import layerRejected from './layer-rejected';
import typeMatch from './type-match.json';
import typeRejected from './type-rejected';

it("should be bundled in the matching layer chunk", () => {
  expect(__filename).toContain("foo.js");
  expect([layerMatch, layerRejected, typeMatch.value, typeRejected]).toEqual([
    "layer match", "layer rejected", "type match", "type rejected"
  ]);
});
