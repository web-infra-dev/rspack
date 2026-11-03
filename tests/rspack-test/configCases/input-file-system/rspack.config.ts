import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import assert from 'node:assert';

let readFileCalled = false;
let stateCalled = false;

export default defineConfig({
  entry: {
    index: './virtual_index.js',
  },
  plugins: [
    {
      apply: (compiler: Compiler) => {
        compiler.hooks.afterCompile.tap('SimpleInputFileSystem', () => {
          assert(readFileCalled, 'readFile should be called');
          assert(stateCalled, 'stat should be called');
        });
        compiler.hooks.beforeCompile.tap('SimpleInputFileSystem', () => {
          // simple file system just works for test case
          compiler.inputFileSystem = {
            readFile(
              _p: string,
              cb: (error: Error | null, content: string) => void,
            ) {
              readFileCalled = true;
              cb(
                null,
                `
								require("./disk.js");
								it("should read file simple file",()=>{
								  expect(1).toBe(1);
							  })`,
              );
            },
            stat(_p: string, cb: (error: Error | null, stat: object) => void) {
              stateCalled = true;
              cb(null, {
                isFile() {
                  return true;
                },
                isDirectory() {
                  return false;
                },
                isSymbolicLink() {
                  return false;
                },
                atimeMs: 1749025843289.6816,
                mtimeMs: 1749025842638.571,
                ctimeMs: 1749025842638.571,
                birthtimeMs: 1749025385767.096,
                size: 1000,
              });
            },
          } as unknown as Compiler['inputFileSystem'];
        });
      },
    },
  ],
  experiments: {
    useInputFileSystem: [/virtual_.*\.js/],
  },
});
