const fs = require('fs');

const adjustedResources = new Set();

module.exports = function (source) {
  return source;
};

module.exports.pitch = function () {
  if (!adjustedResources.has(this.resourcePath)) {
    const reliableMtime = new Date(Date.now() - 3000);
    fs.utimesSync(this.resourcePath, reliableMtime, reliableMtime);
    adjustedResources.add(this.resourcePath);
  }
};
