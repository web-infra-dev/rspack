import cases from "./cases.mjs";

export default cases
  .filter(test => test.error)
  .map(test => new RegExp(test.error));
