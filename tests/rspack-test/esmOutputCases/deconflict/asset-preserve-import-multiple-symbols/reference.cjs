// Keep both assets in module factories through non-ESM incoming dependencies.
module.exports = [
  require('./same-name.asset.mjs'),
  require('./same_name.asset.mjs'),
]
