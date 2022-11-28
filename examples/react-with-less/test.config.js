module.exports = {
  mode: "development",
  entry: {
    main: {
      import: ["./src/index.jsx"],
    }
  },
  define: {
    "process.env.NODE_ENV": "'development'",
  },
  builtins: {
    html: [{}],
  },
  module: {
    rules: [
      {
        test : {
          type: "regexp",
          matcher: '\\.less$'
        },
        type: "asset"
      }
    ]
  }
};
