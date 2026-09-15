it('should preserve context-independent source URLs provided by loaders', function () {
  var fs = require('fs');
  var source = fs.readFileSync(__dirname + '/bundle0.js.map', 'utf-8');
  var map = JSON.parse(source);
  // The loader-provided source URLs are already context-independent
  // (`rspack://` scheme or standard URLs), so they must never be wrapped
  // into doubly-schemed `webpack://rspack:///...` URLs.
  for (const s of map.sources) {
    expect(s.match(/(webpack|rspack):\/\//g)?.length ?? 0).toBeLessThan(2);
  }
  expect(map.sources.some((s) => s.endsWith('/module.js'))).toBe(true);
  // Standard URL sources are passed through verbatim.
  expect(map.sources).toContain('https://cdn.example/https.js');
  expect(map.sources).toContain('data:text/javascript,console.log(1)');
});

require('./module.js');
require('./https.js');
require('./data.js');
