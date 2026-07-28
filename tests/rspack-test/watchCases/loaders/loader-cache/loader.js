const fs = require('fs');

const loaderRuns = {
  left: 0,
  marked: 0,
  right: 0,
};
const adjustedResources = new Set();

module.exports = function (source, sourceMap, additionalData) {
  const { name } = this.getOptions();
  if (name === 'marked') {
    this.addDependency(`${this.rootContext}/trigger.js`);
  }
  loaderRuns[name]++;
  source = source.replace(`__${name.toUpperCase()}__`, loaderRuns[name]);

  if (name === 'right') {
    this.callback(
      null,
      source,
      {
        version: 3,
        sources: ['value.js'],
        names: [],
        mappings: '',
      },
      { right: true },
    );
    return;
  }
  if (name === 'marked') {
    this.callback(null, source, sourceMap, {
      ...additionalData,
      marked: true,
    });
    return;
  }

  return source
    .replace('__SOURCE_MAP__', sourceMap?.version === 3)
    .replace(
      '__ADDITIONAL_DATA__',
      additionalData?.right === true && additionalData?.marked === true,
    );
};

module.exports.pitch = function () {
  const { name } = this.getOptions();
  if (name === 'left' && !adjustedResources.has(this.resourcePath)) {
    const reliableMtime = new Date(Date.now() - 3000);
    fs.utimesSync(this.resourcePath, reliableMtime, reliableMtime);
    adjustedResources.add(this.resourcePath);
  }
};
