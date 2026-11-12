import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { CopyRspackPlugin, type InputFileSystem } from '@rspack/core';

type Callback<T> = (error: NodeJS.ErrnoException | null, result?: T) => void;

function normalizePath(filePath: string) {
  const normalized = filePath.replace(/\\/g, '/');
  return normalized.length > 1 ? normalized.replace(/\/+$/, '') : normalized;
}

const root = import.meta.dirname;
const virtualRoot = normalizePath(path.join(root, 'virtual'));
const files = new Map([
  [`${virtualRoot}/direct.txt`, 'direct from js input fs'],
  [`${virtualRoot}/.env`, 'dotfile from js input fs\n'],
  [`${virtualRoot}/nested/file.txt`, 'nested from js input fs\n'],
]);

function createStats(filePath: string) {
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
    size: isFile ? Buffer.byteLength(files.get(normalizedPath)!) : 0,
    mode: isFile ? 0o100644 : 0o040755,
  };
}

function isVirtualPath(filePath: string) {
  const normalizedPath = normalizePath(filePath);
  return (
    normalizedPath === virtualRoot ||
    normalizedPath.startsWith(`${virtualRoot}/`)
  );
}

function createInputFileSystem(originalInputFileSystem: InputFileSystem) {
  const inputFileSystem: InputFileSystem = Object.create(
    originalInputFileSystem,
  );

  Object.assign(inputFileSystem, {
    readFile(filePath: string, callback: Callback<Buffer>) {
      if (!isVirtualPath(filePath)) {
        return originalInputFileSystem.readFile(filePath, callback);
      }
      const normalizedPath = normalizePath(filePath);
      if (files.has(normalizedPath)) {
        callback(null, Buffer.from(files.get(normalizedPath)!));
      } else {
        callback(
          Object.assign(new Error(`ENOENT: ${filePath}`), { code: 'ENOENT' }),
        );
      }
    },
    readdir(dirPath: string, callback: Callback<string[]>) {
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
    stat(filePath: string, callback: Callback<ReturnType<typeof createStats>>) {
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
    lstat(
      filePath: string,
      callback: Callback<ReturnType<typeof createStats>>,
    ) {
      this.stat(filePath, callback);
    },
    realpath(filePath: string, callback: Callback<string>) {
      if (!isVirtualPath(filePath)) {
        return originalInputFileSystem.realpath!(filePath, callback);
      }
      callback(null, filePath);
    },
  });

  return inputFileSystem;
}

export default defineConfig({
  entry: './index.js',
  target: 'node',
  experiments: {
    useInputFileSystem: [/copy-plugin-input-file-system[\\/]virtual/],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const inputFileSystem = createInputFileSystem(
          compiler.inputFileSystem!,
        );
        compiler.inputFileSystem = inputFileSystem;
        compiler.hooks.beforeCompile.tap('CopyPluginInputFileSystem', () => {
          compiler.inputFileSystem = inputFileSystem;
        });
      },
    }),
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
});
