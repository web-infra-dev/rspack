let count = 0;

module.exports = function () {
  return `export default ${++count};`;
};
