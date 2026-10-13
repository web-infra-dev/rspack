module.exports = function () {
  return `export default ${JSON.stringify({ query: this.resourceQuery, fragment: this.resourceFragment })}`;
};
