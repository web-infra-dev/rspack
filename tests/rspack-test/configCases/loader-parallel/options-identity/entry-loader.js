const { randomUUID } = require('node:crypto');
module.exports = function () {
  const generation = randomUUID();
  return `module.exports = [${Array.from({ length: 128 }, (_, i) => `require('./value.js?${generation}:${i}')`).join(',')}];`;
};
