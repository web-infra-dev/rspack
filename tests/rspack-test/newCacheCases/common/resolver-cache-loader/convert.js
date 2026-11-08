module.exports = function (source) {
  return `export default ${JSON.stringify(`js:${source.trim()}`)}`;
};
