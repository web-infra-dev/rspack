const cases = [
  ['defaults', require('./style.less?defaults'), require('./style.less?defaults&reference')],
  ['relative-urls-disabled', require('./style.less?relative-urls-disabled'), require('./style.less?relative-urls-disabled&reference')],
  ['rewrite-urls-overridden', require('./style.less?rewrite-urls-overridden'), require('./style.less?rewrite-urls-overridden&reference')],
  ['rewrite-urls-off', require('./style.less?rewrite-urls-off'), require('./style.less?rewrite-urls-off&reference')],
  ['rewrite-urls-local', require('./style.less?rewrite-urls-local'), require('./style.less?rewrite-urls-local&reference')],
  ['numeric-math', require('./style.less?numeric-math'), require('./style.less?numeric-math&reference')],
  ['strict-math', require('./style.less?strict-math'), require('./style.less?strict-math&reference')],
  ['line-numbers', require('./style.less?line-numbers'), require('./style.less?line-numbers&reference')],
  ['banner', require('./style.less?banner'), require('./style.less?banner&reference')],
  ['skip-imports', require('./style.less?skip-imports'), require('./style.less?skip-imports&reference')],
  ['paths', require('./paths.less?paths'), require('./paths.less?paths&reference')],
  ['filename', require('./style.less?filename'), require('./style.less?filename&reference')],
  ['compiler-map', require('./style.less?compiler-map'), require('./style.less?compiler-map&reference')],
  ['loader-map', require('./style.less?loader-map'), require('./style.less?loader-map&reference')],
  ['loader-map-overrides-false', require('./style.less?loader-map-overrides-false'), require('./style.less?loader-map-overrides-false&reference')],
];

for (const [name, builtin, reference] of cases) {
  it(`matches less-loader compiler options: ${name}`, () => {
    const actual = JSON.parse(builtin);
    const expected = JSON.parse(reference);
    expect(actual.css).toBe(expected.css);
    if (expected.map) {
      expect(actual.map).toBeTruthy();
      expect(actual.map.sources).toEqual(expected.map.sources);
      expect(actual.map.sourcesContent).toEqual(expected.map.sourcesContent);
      expect(actual.map.file).toBe(expected.map.file);
      expect(actual.map.sourceRoot || '').toBe(expected.map.sourceRoot || '');
    } else {
      expect(actual.map).toBeFalsy();
    }
  });
}
