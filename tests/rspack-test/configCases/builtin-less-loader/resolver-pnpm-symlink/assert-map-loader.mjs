export default function (css, map) {
  const { realTokens } = this.getOptions();
  const index = map.sources.indexOf(realTokens.replace(/\\/g, '/'));
  expect(index).toBeGreaterThanOrEqual(0);
  expect(map.sourcesContent[index]).toContain('@color: purple;');
  return css;
}
