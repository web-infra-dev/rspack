module.exports = async function (source) {
  await Promise.resolve();
  this.emitFile('loader.txt', 'loader');
  return source;
};
