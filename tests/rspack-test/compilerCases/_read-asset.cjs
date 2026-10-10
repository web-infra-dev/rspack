const path = require('node:path');

// After emit the compilation only retains SizeOnlySource; inspect emitted bytes.
exports.readAsset = (compilation, name) => {
  if (!compilation.getAsset(name)) return undefined;
  return compilation.compiler.outputFileSystem.readFileSync(
    path.join(compilation.outputOptions.path, name.split('?')[0]),
    'utf-8',
  );
};
