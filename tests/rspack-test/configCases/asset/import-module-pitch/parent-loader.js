module.exports = async function () {
  const { default: url } = await this.importModule(
    `!!${require.resolve('./pitch-loader')}!${require.resolve('./target.txt')}`,
    { publicPath: '' },
  );
  return `module.exports = ${JSON.stringify(url)};`;
};
