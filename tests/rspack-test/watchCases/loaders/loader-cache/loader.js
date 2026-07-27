const loaderRuns = {
  left: 0,
  marked: 0,
  right: 0,
};

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
