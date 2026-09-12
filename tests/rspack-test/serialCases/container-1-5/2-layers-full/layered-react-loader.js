module.exports = function (source) {
  const transformed = source.replace('No Layer', 'This is layered react');
  return transformed;
};
