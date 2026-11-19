const path = require('node:path');

module.exports = {
  findBundle(_index, options) {
    return path.join(options.output.path, 'main.js');
  },
};
