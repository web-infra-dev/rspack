module.exports = function (source, sourceMap, additionalData) {
  expect(source).toBeNull();
  expect(sourceMap).toBeUndefined();
  expect(additionalData).toBeUndefined();
  return 'module.exports = "empty";';
};
