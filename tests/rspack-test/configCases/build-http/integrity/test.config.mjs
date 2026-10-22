import cases from "./cases.ts";

export default {
  isolateSource: true,
  findBundle(index) {
    return cases[index].error ? [] : `./bundle${index}.js`;
  },
};
