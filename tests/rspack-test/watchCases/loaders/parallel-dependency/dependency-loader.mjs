import fs from "node:fs";
import path from "node:path";

/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
  if (!this.parallel) {
    throw new Error('dependency-loader must run in a worker');
  }
  const dependency = path.join(this.context, this.getOptions().dependency);
  this.addDependency(dependency);
  return source.replace(
    '__DEPENDENCY_VALUE__',
    JSON.stringify(fs.readFileSync(dependency, 'utf-8').trim()),
  );
};
