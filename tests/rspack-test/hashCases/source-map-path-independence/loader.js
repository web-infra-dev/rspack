module.exports = function (source) {
  const { sandbox, changeCode } = this.getOptions();
  const code = changeCode ? source.replace('log(1)', 'log(2)') : source;
  this.callback(null, code, {
    version: 3,
    file: `${sandbox}/dist/mod.js`,
    sources: [`${sandbox}/app/src/mod.ts`],
    sourceRoot: sandbox,
    names: [],
    sourcesContent: [code],
    mappings: 'AAAA',
  });
};
