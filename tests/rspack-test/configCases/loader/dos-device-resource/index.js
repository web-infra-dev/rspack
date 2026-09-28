const parsedResources = require('./resource');

it('should parse DOS device paths without treating the prefix as a query', () => {
  expect(parsedResources).toEqual([
    {
      resourcePath: String.raw`\\?\C:\very\long\resource.js`,
      resourceQuery: '?resource-query',
      resourceFragment: '#fragment',
    },
    {
      resourcePath: String.raw`\\.\C:\device\resource.js`,
      resourceQuery: '?resource-query',
      resourceFragment: '#fragment',
    },
    {
      resourcePath: String.raw`\\?\UNC\server\share\resource.js`,
      resourceQuery: '?resource-query',
      resourceFragment: '#fragment',
    },
    {
      resourcePath: String.raw`\\?\C:\escaped#name.js`,
      resourceQuery: '?resource-query',
      resourceFragment: '#fragment',
    },
  ]);
});
