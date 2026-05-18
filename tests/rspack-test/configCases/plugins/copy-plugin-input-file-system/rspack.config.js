const path = require('path');
const { CopyRspackPlugin } = require('@rspack/core');

function normalizePath(filePath) {
  const normalized = filePath.replace(/\\/g, '/');
  return normalized.length > 1 ? normalized.replace(/\/+$/, '') : normalized;
}

const root = __dirname;
const virtualRoot = normalizePath(path.join(root, 'virtual'));
const files = new Map([
  [`${virtualRoot}/direct.txt`, 'direct from js input fs'],
  [`${virtualRoot}/.env`, 'dotfile from js input fs\n'],
  [`${virtualRoot}/nested/file.txt`, 'nested from js input fs\n'],
]);

function createStats(filePath) {
  const normalizedPath = normalizePath(filePath);
  const isFile = files.has(normalizedPath);
  const isDirectory =
    normalizedPath === virtualRoot ||
    normalizedPath === `${virtualRoot}/nested`;

  return {
    isFile: () => isFile,
    isDirectory: () => isDirectory,
    isSymbolicLink: () => false,
    atimeMs: 0,
    mtimeMs: 0,
    ctimeMs: 0,
    birthtimeMs: 0,
    size: isFile ? Buffer.byteLength(files.get(normalizedPath)) : 0,
    mode: isFile ? 0o100644 : 0o040755,
  };
}

function isVirtualPath(filePath) {
  const normalizedPath = normalizePath(filePath);
  return (
    normalizedPath === virtualRoot ||
    normalizedPath.startsWith(`${virtualRoot}/`)
  );
}

function createInputFileSystem(originalInputFileSystem) {
  const inputFileSystem = Object.create(originalInputFileSystem);

  Object.assign(inputFileSystem, {
    readFile(filePath, callback) {
      if (!isVirtualPath(filePath)) {
        return originalInputFileSystem.readFile(filePath, callback);
      }
      const normalizedPath = normalizePath(filePath);
      if (files.has(normalizedPath)) {
        callback(null, Buffer.from(files.get(normalizedPath)));
      } else {
        callback(
          Object.assign(new Error(`ENOENT: ${filePath}`), { code: 'ENOENT' }),
        );
      }
    },
    readdir(dirPath, callback) {
      if (!isVirtualPath(dirPath)) {
        return originalInputFileSystem.readdir(dirPath, callback);
      }
      const normalizedPath = normalizePath(dirPath);
      if (normalizedPath === virtualRoot) {
        callback(null, ['direct.txt', '.env', 'nested']);
      } else if (normalizedPath === `${virtualRoot}/nested`) {
        callback(null, ['file.txt']);
      } else {
        callback(
          Object.assign(new Error(`ENOENT: ${dirPath}`), { code: 'ENOENT' }),
        );
      }
    },
    stat(filePath, callback) {
      if (!isVirtualPath(filePath)) {
        return originalInputFileSystem.stat(filePath, callback);
      }
      const stats = createStats(filePath);
      if (stats.isFile() || stats.isDirectory()) {
        callback(null, stats);
      } else {
        callback(
          Object.assign(new Error(`ENOENT: ${filePath}`), { code: 'ENOENT' }),
        );
      }
    },
    lstat(filePath, callback) {
      this.stat(filePath, callback);
    },
    realpath(filePath, callback) {
      if (!isVirtualPath(filePath)) {
        return originalInputFileSystem.realpath(filePath, callback);
      }
      callback(null, filePath);
    },
  });

  return inputFileSystem;
}

module.exports = {
  entry: './index.js',
  target: 'node',
  experiments: {
    useInputFileSystem: [/copy-plugin-input-file-system[\\/]virtual/],
  },
  plugins: [
    {
      apply(compiler) {
        const inputFileSystem = createInputFileSystem(compiler.inputFileSystem);
        compiler.inputFileSystem = inputFileSystem;
        compiler.hooks.beforeCompile.tap('CopyPluginInputFileSystem', () => {
          compiler.inputFileSystem = inputFileSystem;
        });
      },
    },
    new CopyRspackPlugin({
      patterns: [
        {
          from: 'virtual/direct.txt',
          to: 'copied/direct.txt',
        },
        {
          from: 'virtual/**/*',
          to: 'copied/glob',
          globOptions: {
            dot: true,
          },
        },
      ],
    }),
  ],
  output: {
    clean: true,
  },
};
