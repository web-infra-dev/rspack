import path from "node:path";

export default function (source) {
  const name = source.trim();
  const methods = {
    file: "addDependency",
    context: "addContextDependency",
    missing: "addMissingDependency",
    build: "addBuildDependency"
  };
  for (const [kind, method] of Object.entries(methods)) {
    for (const dependency of ["shared", name]) {
      this[method](path.join(this.context, "tracked", `${dependency}.${kind}`));
    }
  }
  return `export default ${JSON.stringify(name)};`;
}
