module.exports = function () {
  const request = this.resourceQuery === '?first' ? 'query-first' : 'query-second';
  return `import shared from ${JSON.stringify(request)}; export default shared;`;
};
