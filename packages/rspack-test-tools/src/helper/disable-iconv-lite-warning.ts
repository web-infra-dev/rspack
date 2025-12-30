export function disableIconvLiteWarning() {
  for (const [path, mod] of Object.entries(require.cache)) {
    if (
      path.includes('iconv-lite') &&
      typeof mod?.exports === 'object' &&
      typeof mod.exports.decode === 'function'
    ) {
      mod.exports.skipDecodeWarning = true;
    }
  }
}
