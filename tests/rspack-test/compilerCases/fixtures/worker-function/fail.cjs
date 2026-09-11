module.exports = async (_data, options) => {
  if (options.kind === 'crash') process.exit(1);
  if (options.kind === 'throw') throw new Error('worker function failure');
  if (options.kind === 'invalid') return {};
  if (options.kind === 'conversion') _data.request = {};
};
