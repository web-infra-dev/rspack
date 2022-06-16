const cssLoaderPath = require.resolve("css-loader").replace(/\\/g, "/");

module.exports = [
  "",
  "WARNING in chunk styles [mini-css-extract-plugin]",
  "Conflicting order. Following module has been added:",
  ` * css ${cssLoaderPath}!./e2.css`,
  "despite it was not able to fulfill desired ordering with these modules:",
  ` * css ${cssLoaderPath}!./e1.css`,
  "   - couldn't fulfill desired order of chunk group(s) entry2",
  "   - while fulfilling desired order of chunk group(s) entry1",
].join("\n");
