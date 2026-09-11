module.exports = async (data, options) => {
  if (data.request === './virtual') data.request = options.to;
};
