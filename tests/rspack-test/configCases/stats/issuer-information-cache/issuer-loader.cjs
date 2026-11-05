module.exports = async function (source) {
  const { first } = await this.importModule('./first.js');
  if (first() !== 20) throw new Error('expected the build-time issuer graph to execute');
  return source;
};
