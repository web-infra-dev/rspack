module.exports = {
  findBundle(index) {
    return index === 0 ? ['./used-0.js', './unused-0.js'] : './unused-1.js';
  },
};
