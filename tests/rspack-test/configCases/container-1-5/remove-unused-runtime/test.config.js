module.exports = {
  findBundle(index, options) {
    return ['filtered', 'untouched'].map(
      (entry) => `./${options.name}/${entry}.js`,
    );
  },
};
