module.exports = (env) => {
  console.log(env);
  const { environment, app, file } = env;
  const customName = file && file.name && file.name.is && file.name.is.this;
  const appTitle = app && app.title;
  if (environment === "production") {
    return {
      entry: "./a.js",
      output: {
        filename: `${customName ? customName : appTitle}.js`,
      },
    };
  }
  if (environment === "multipleq") {
    const { file } = env;
    return {
      entry: "./a.js",
      output: {
        filename: `${file}.js`,
      },
    };
  }
  if (environment === "dot") {
    const file = env["name."];
    return {
      entry: "./a.js",
      output: {
        filename: `${file}.js`,
      },
    };
  }
  return {};
};
