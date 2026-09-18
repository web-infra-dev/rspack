import cases from "./cases.mjs";

export default {
  isolateSource: true,
  findBundle(index) {
    return cases[index].error ? [] : `./bundle${index}.js`;
  },
};
