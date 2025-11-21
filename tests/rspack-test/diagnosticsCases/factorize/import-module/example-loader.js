module.exports = async function (content, map, meta) {
  const cb = this.async();

  try {
    await this.importModule('import-module-example!./non-existent-module');
  } catch (err) {
    console.log('loaderContext.importModule', err);
    // this will never happen because of the rust panic
    return cb(new Error('import-module-example!./non-existent-module'));
  }

  return cb(null, content, map, meta);
};
