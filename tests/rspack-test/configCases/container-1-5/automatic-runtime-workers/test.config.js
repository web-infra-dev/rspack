module.exports = {
  findBundle(index, options) {
    return ['main', 'local-a', 'local-b', 'remote', 'consume'].map(
      (entry) => `./${options.name}/${entry}.js`,
    );
  },
};
