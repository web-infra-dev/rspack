module.exports = {
  findBundle(index, options) {
    return ['plain', 'remote', 'consume'].map((entry) => `./${options.name}/${entry}.js`);
  },
};
