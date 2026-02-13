module.exports = function (source) {
  const issuerLayer = this._module?.layer;
  return source.replace('No Layer', `${issuerLayer}`);
};
