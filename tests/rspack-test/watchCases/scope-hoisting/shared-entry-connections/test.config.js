module.exports = {
  ignoreNotFriendlyForIncrementalWarnings: true,
  findBundle() {
    return ['a.js', 'b.js'];
  },
};
