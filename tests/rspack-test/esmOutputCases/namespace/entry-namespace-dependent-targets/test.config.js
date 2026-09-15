module.exports = {
  findBundle(_index, options) {
    return `main.${options.name}.mjs`;
  },
  snapshotFileFilter: file => !file.startsWith('runtime.'),
};
