import fs from "node:fs";
import path from "node:path";

export default function () {
  const dependency = path.join(import.meta.dirname, 'build-dependency.js');
  this.addBuildDependency(dependency);
  return `export default ${JSON.stringify(fs.readFileSync(dependency, 'utf-8').trim())};`;
};
