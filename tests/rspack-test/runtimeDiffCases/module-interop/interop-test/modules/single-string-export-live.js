module.exports = "single-outdated";

Promise.resolve().then(() => {
  module.exports = "single";
});
