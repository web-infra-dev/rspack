module.exports = function () {
  const size = this.resourceQuery === '?small'
    ? 6000
    : this.resourceQuery === '?resource' ? 9000 : 6016;
  return 'x'.repeat(size);
};
