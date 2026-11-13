export default {
  findBundle(index) {
    return [`main${index}.${index ? 'mjs' : 'js'}`];
  },
};
