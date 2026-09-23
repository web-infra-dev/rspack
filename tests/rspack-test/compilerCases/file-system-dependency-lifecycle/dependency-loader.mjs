import path from 'node:path';

export default function (source) {
  this.cacheable(false);
  const shared = path.join(this.context, '共享依赖🦀');
  for (const method of [
    'addDependency',
    'addContextDependency',
    'addMissingDependency',
    'addBuildDependency',
  ]) {
    this[method](shared);
    this[method](shared);
  }
  return source;
}
