module.exports = {
  findBundle(index) {
    return ['main', 'secondary', 'third'].map((name) => `./${index}/${name}.js`);
  },
};
