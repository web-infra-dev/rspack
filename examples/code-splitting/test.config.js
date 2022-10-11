module.exports = {
  mode: "development",
  entry: {
    main: "./src/index.js",
  },
  // output: {
  //   publicPath: "http://localhost:3000",
  // },
  define: {
    "process.env.NODE_ENV": "development",
  },
	target: ["web", "es2020"],
  builtins: {
    html: [{}],
    polyfill: false
  },
};
