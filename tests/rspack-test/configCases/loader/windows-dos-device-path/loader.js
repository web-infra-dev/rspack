module.exports = function (source) {
  if (!this.resourcePath.startsWith('\\\\?\\')) {
    throw new Error(`Expected a DOS device resource path, got ${this.resourcePath}`);
  }
  if (!/^[a-zA-Z]:[\\/]/.test(this.loaders[this.loaderIndex].path)) {
    throw new Error(
      `Expected a regular drive loader path, got ${this.loaders[this.loaderIndex].path}`,
    );
  }
  if (this.resourceQuery !== '?resource-query') {
    throw new Error(`Unexpected resource query: ${this.resourceQuery}`);
  }
  if (this.resourceFragment !== '#fragment') {
    throw new Error(`Unexpected resource fragment: ${this.resourceFragment}`);
  }
  if (this.query !== '?loader-query') {
    throw new Error(`Unexpected loader query: ${this.query}`);
  }
  return source;
};
