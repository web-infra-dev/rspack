export default {
  findBundle(index) {
    return `./${index === 0 ? 'lower' : 'upper'}/0.js`;
  },
};
