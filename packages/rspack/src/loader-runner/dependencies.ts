import type { JsLoaderContext } from '@rspack/binding';

export type LoaderDependencies = JsLoaderContext['dependencies'];

const DEPENDENCY_KEYS = [
  'fileDependencies',
  'contextDependencies',
  'missingDependencies',
  'buildDependencies',
] as const;

type DependencyKey = (typeof DEPENDENCY_KEYS)[number];

const createDependencies = (): LoaderDependencies => ({
  fileDependencies: [],
  contextDependencies: [],
  missingDependencies: [],
  buildDependencies: [],
});

const clear = (dependencies: string[]) => {
  dependencies.length = 0;
};

export class LoaderDependenciesState {
  readonly existing: LoaderDependencies;
  readonly added = createDependencies();
  readonly removed = createDependencies();

  constructor(existing: LoaderDependencies) {
    this.existing = existing;
  }

  resetChanges() {
    for (const key of DEPENDENCY_KEYS) {
      clear(this.added[key]);
      clear(this.removed[key]);
    }
  }

  mergeChanges() {
    for (const key of DEPENDENCY_KEYS) {
      if (this.added[key].length === 0 && this.removed[key].length === 0) {
        continue;
      }
      const dependencies = this.get(key);
      clear(this.existing[key]);
      this.existing[key].push(...dependencies);
    }
    this.resetChanges();
  }

  addDependencies(dependencies: LoaderDependencies) {
    for (const key of DEPENDENCY_KEYS) {
      for (const dependency of dependencies[key]) {
        this.add(key, dependency);
      }
    }
  }

  addFile(dependency: string) {
    this.add('fileDependencies', dependency);
  }

  addContext(dependency: string) {
    this.add('contextDependencies', dependency);
  }

  addMissing(dependency: string) {
    this.add('missingDependencies', dependency);
  }

  addBuild(dependency: string) {
    this.add('buildDependencies', dependency);
  }

  fileDependencies() {
    return this.get('fileDependencies');
  }

  contextDependencies() {
    return this.get('contextDependencies');
  }

  missingDependencies() {
    return this.get('missingDependencies');
  }

  clearDependencies() {
    this.clear('fileDependencies');
    this.clear('contextDependencies');
    this.clear('missingDependencies');
  }

  private add(key: DependencyKey, dependency: string) {
    const removed = this.removed[key];
    for (let index = removed.length - 1; index >= 0; index--) {
      if (removed[index] === dependency) removed.splice(index, 1);
    }
    if (!this.added[key].includes(dependency)) {
      this.added[key].push(dependency);
    }
  }

  private get(key: DependencyKey) {
    const removed = new Set(this.removed[key]);
    return Array.from(
      new Set(
        this.existing[key]
          .filter((dependency) => !removed.has(dependency))
          .concat(this.added[key]),
      ),
    );
  }

  private clear(key: DependencyKey) {
    clear(this.removed[key]);
    this.removed[key].push(...this.existing[key]);
    clear(this.added[key]);
  }
}
