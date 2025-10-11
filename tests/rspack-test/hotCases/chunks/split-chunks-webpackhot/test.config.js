module.exports = {
  documentType: "fake",
  findBundle: function (i, options) {
    return ["main.js", "0.js"];
  }
};