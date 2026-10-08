module.exports = {
  findBundle(index, options) {
    return options.output.filename;
  },
  moduleScope(scope) {
    // The web runner supplies both APIs; a real document has no importScripts.
    delete scope.importScripts;
  }
};
