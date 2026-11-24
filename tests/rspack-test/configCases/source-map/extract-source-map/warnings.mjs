export default [
  /Failed to read '.*[/\\]no-source-map\.map' while loading the source map:/,
  /Failed to parse source map '.*[/\\]invalid\.map':/,
  /Failed to decode source map data URL in 'invalid-base64\.js': invalid Base64 data\s*$/,
  /Failed to decode source map data URL in 'invalid-utf8\.js\.map': decoded content is not valid UTF-8\s*$/,
  /Failed to decode source map data URL in 'malformed-data-url\.js': missing comma before the data\s*$/,
];
