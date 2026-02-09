const base = require('./bench-ts-react.config.cjs');

module.exports = {
  ...base,
  experiments: {
    ...(base.experiments ?? {}),
    outputModule: true,
  },
  output: {
    ...(base.output ?? {}),
    module: true,
  },
};
