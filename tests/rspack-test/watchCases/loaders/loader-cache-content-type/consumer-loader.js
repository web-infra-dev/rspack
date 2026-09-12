const runs = new Map();

module.exports = function (source) {
  const key = `${this.getOptions().name}:${this.resource}`;
  const count = (runs.get(key) || 0) + 1;
  runs.set(key, count);
  return `module.exports = ${JSON.stringify({ content: source, runs: count })};`;
};
