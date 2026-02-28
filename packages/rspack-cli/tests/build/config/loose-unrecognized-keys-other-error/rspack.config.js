module.exports = {
  entry: './index.js',
  context: './context', // relative path will fail
  _additionalProperty: 'test',
};
