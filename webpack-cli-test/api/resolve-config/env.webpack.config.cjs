module.exports = function (env) {
  const configName = env.name;
  return {
    name: configName,
    mode: env.test ? "staging" : "production",
  };
};
