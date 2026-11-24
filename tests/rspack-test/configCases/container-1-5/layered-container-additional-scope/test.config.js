module.exports = {
  findBundle(_index, options) {
    return `./${options.output.uniqueName}-host.js`;
  },
};
