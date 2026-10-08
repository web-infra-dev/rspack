export default {
  findBundle(index) {
    return ['main', 'jsonParse', 'array', 'javascript', 'conditional', 'loop'].map(
      name => `${name}${index}.mjs`,
    );
  },
};
