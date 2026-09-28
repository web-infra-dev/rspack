import path from 'node:path';

// Mimics tailwind-style css generation: the stylesheet depends on other
// source files, so editing them rebuilds the css module even when the
// emitted css stays byte-identical.
export default function (source) {
  this.addDependency(path.resolve(import.meta.dirname, 'src/dep.txt'));
  return source;
}
