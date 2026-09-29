import cases from "./cases.ts";

export default cases
  .filter(test => test.error)
  .map(test => new RegExp(test.error));
