const cases = require('./cases');

module.exports = {
  isolateSource: true,
  findBundle(index) {
    return cases[index].error ? [] : `./bundle${index}.js`;
  },
};
