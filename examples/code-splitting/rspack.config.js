module.exports = {
  mode: "development",
  entry: {
    main: {
      import: ["./index.js"],
    }
  },
  define: {
    "process.env.NODE_ENV": "'development'",
  },
  builtins: {
    html: [{}],
  },
};
