export default {
  findBundle(index) {
    return ['main', 'jsonParse', 'array', 'javascript'].map(
      name => `${name}${index}.mjs`,
    );
  },
};
