import path from "node:path";

export default function (source) {
  this.cacheable(false);
  for (const method of [
    "addDependency",
    "addContextDependency",
    "addMissingDependency",
    "addBuildDependency",
  ]) {
    const shared = path.join(this.context, "共享依赖🦀");
    this[method](shared);
    this[method](shared);
  }
  return source;
}
