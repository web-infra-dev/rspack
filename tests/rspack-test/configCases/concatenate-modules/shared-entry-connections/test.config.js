module.exports = {
  findBundle(index, options) {
    return ['entry1', 'entry2'].map((name) =>
      options.output.filename.replace('[name]', name),
    );
  },
};
