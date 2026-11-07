export default {
  isolateSource: true,
  findBundle(index) {
    return index < 4 ? `./bundle${index}.js` : [];
  },
};
