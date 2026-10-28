export default {
  findBundle(index) {
    return ['main', 'secondary', 'third'].map((name) => `./${index}/${name}.js`);
  },
};
