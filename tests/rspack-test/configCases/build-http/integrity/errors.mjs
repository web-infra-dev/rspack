import cases from "./cases.js";

export default cases
  .filter(test => test.error)
  .map(test => new RegExp(test.error));
