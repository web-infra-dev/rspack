// For 'no-strict.js', JSON named export warnings are ignored intentionally, other bundlers support it without warning.
// For 'strict.mjs', it's strict ESM, so JSON named export are reported as error.
module.exports = [
  [
    /Can't import the named export 'aa' \(imported as 'aa'\) from default-exporting module \(only default export is available\)/,
    { moduleName: /strict\.mjs/ },
  ],
];
