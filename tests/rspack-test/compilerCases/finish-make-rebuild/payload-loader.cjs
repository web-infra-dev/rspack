module.exports = function () {
  this.cacheable(false);
  const owner = this.getOptions().owner;
  const value = owner === 'main' ? this.rebuildState.phase : 100;
  this.emitFile(`${owner}-${value}.txt`, `${value}`);
  this.emitWarning(new Error(`${owner} warning ${value}`));
  return `export default ${value};`;
};
