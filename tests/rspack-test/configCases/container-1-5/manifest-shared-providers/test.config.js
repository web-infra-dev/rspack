module.exports = {
  findBundle(index) {
    return index === 0 ? './layered/main.js' : [];
  },
};
