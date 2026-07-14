module.exports = ({ rspackOptions, target }) =>
  target === "web" && rspackOptions?.experiments?.runtimeMode !== "rspack";
