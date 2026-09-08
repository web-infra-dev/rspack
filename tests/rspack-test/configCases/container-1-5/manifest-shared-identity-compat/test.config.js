module.exports = {
  findBundle(index) {
    return index === 0 ? './analyzed/main.js' : [];
  },
};
