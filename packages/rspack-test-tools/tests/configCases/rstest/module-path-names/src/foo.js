export const file1 = __filename
export const file2 = __filename
export const metaFile1 = import.meta.filename
export const typeofMetaFile = typeof import.meta.filename

export const dir1 =  __dirname
export const dir2 =  __dirname
export const metaDir1 = import.meta.dirname
export const typeofMetaDir = typeof import.meta.dirname

if (import.meta.filename.indexOf("foo.js") === -1) {
  require("fail")
};

if (import.meta.dirname.indexOf("src") === -1) {
  require("fail")
};

