import cases from "./cases.js";

export default {
  isolateSource: true,
  findBundle(index) {
    return cases[index].error ? [] : `./bundle${index}.js`;
  },
};
