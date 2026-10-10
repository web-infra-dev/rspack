const path = require('node:path');

// After emit the compilation only retains SizeOnlySource; inspect emitted bytes.
exports.readAsset = (compilation, name) => {
  if (!compilation.getAsset(name)) return undefined;
  const outputPath = compilation.outputOptions.path;
  // The mock output filesystem keeps POSIX paths even on Windows.
  const pathUtils = path.posix.isAbsolute(outputPath) ? path.posix : path.win32;
  return compilation.compiler.outputFileSystem.readFileSync(
    pathUtils.join(outputPath, name.split('?')[0]),
    'utf-8',
  );
};
