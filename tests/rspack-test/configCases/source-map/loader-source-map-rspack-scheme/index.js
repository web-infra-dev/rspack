it('should not prefix rspack:// source URLs provided by loaders again', function () {
  var fs = require('fs');
  var source = fs.readFileSync(__dirname + '/bundle0.js.map', 'utf-8');
  var map = JSON.parse(source);
  // The loader-provided `rspack:///./module.js` source is already
  // context-independent, so it must never be wrapped into a doubly-schemed
  // `webpack://rspack:///...` URL.
  for (const s of map.sources) {
    expect(s.match(/(webpack|rspack):\/\//g)?.length ?? 0).toBeLessThan(2);
  }
  expect(map.sources.some((s) => s.endsWith('/module.js'))).toBe(true);
});

require('./module.js');
