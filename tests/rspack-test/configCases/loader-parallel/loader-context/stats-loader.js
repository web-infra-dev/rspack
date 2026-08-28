/** @type {import('@rspack/core').LoaderDefinition} */
module.exports = function () {
  const callback = this.async();
  const syncIsFile = this.fs.statSync(this.resourcePath).isFile();
  this.fs.stat(this.resourcePath, (error, stats) => {
    callback(error, `module.exports = ${syncIsFile && stats.isFile()}`);
  });
};
