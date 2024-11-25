const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new rspack.EntryPlugin(__dirname, "./a.js", {
      filename: () => "pages/[name].js",
      name: "a",
    }),
  ],
};
