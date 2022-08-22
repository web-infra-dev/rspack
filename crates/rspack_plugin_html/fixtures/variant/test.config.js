module.exports = {
  entry: {
    index: "./index.js",
  },
  builtins: {
    html: [
      {
        filename: "output.html",
        template: "input.html",
        inject: "head",
        script_loading: "blocking",
        sri: "sha512",
      },
    ],
  },
};
