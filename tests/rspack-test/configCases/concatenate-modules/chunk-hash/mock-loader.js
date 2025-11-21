let mocked = false;
module.exports = function () {
  if (mocked) {
    return `export const m2 = function () { return "content2" };`;
  } else {
    mocked = true;
    return `export const m2 = function () { return "content1" };`;
  }
};