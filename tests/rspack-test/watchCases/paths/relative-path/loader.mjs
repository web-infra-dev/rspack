import path from "node:path";

export default function (source) {
  const depPath = path.resolve(this.context, '_module.js');
  const currentWorkDir = path.resolve('./');
  const relativePath = path.relative(currentWorkDir, depPath);
  this.addDependency(relativePath);
  return source;
}
