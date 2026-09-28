import type { JsLoaderContext } from '@rspack/binding';

export type LoaderDependencies = JsLoaderContext['dependencies'];

const DEPENDENCY_KEYS = [
  'fileDependencies',
  'contextDependencies',
  'missingDependencies',
  'buildDependencies',
] as const;

type DependencyKey = (typeof DEPENDENCY_KEYS)[number];

type DependencySets = Record<DependencyKey, Set<string>>;

const createSets = (): DependencySets => ({
  fileDependencies: new Set(),
  contextDependencies: new Set(),
  missingDependencies: new Set(),
  buildDependencies: new Set(),
});

const setsFrom = (dependencies: LoaderDependencies): DependencySets => ({
  fileDependencies: new Set(dependencies.fileDependencies),
  contextDependencies: new Set(dependencies.contextDependencies),
  missingDependencies: new Set(dependencies.missingDependencies),
  buildDependencies: new Set(dependencies.buildDependencies),
});

const arraysFrom = (sets: DependencySets): LoaderDependencies => ({
  fileDependencies: Array.from(sets.fileDependencies),
  contextDependencies: Array.from(sets.contextDependencies),
  missingDependencies: Array.from(sets.missingDependencies),
  buildDependencies: Array.from(sets.buildDependencies),
});

export class LoaderDependenciesState {
  readonly existing: LoaderDependencies;
  readonly #existingSets: DependencySets;
  readonly #addedSets = createSets();
  readonly #removedSets = createSets();

  constructor(existing: LoaderDependencies) {
    this.existing = existing;
    this.#existingSets = setsFrom(existing);
  }

  get added(): LoaderDependencies {
    return arraysFrom(this.#addedSets);
  }

  get removed(): LoaderDependencies {
    return arraysFrom(this.#removedSets);
  }

  resetChanges() {
    for (const key of DEPENDENCY_KEYS) {
      this.#addedSets[key].clear();
      this.#removedSets[key].clear();
    }
  }

  mergeChanges() {
    for (const key of DEPENDENCY_KEYS) {
      const added = this.#addedSets[key];
      const removed = this.#removedSets[key];
      if (added.size === 0 && removed.size === 0) {
        continue;
      }
      const existing = this.#existingSets[key];
      for (const dependency of removed) {
        existing.delete(dependency);
      }
      for (const dependency of added) {
        existing.add(dependency);
      }
      const target = this.existing[key];
      target.length = 0;
      for (const dependency of existing) {
        target.push(dependency);
      }
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
    this.#removedSets[key].delete(dependency);
    this.#addedSets[key].add(dependency);
  }

  private get(key: DependencyKey) {
    const removed = this.#removedSets[key];
    const dependencies = new Set<string>();
    for (const dependency of this.#existingSets[key]) {
      if (!removed.has(dependency)) {
        dependencies.add(dependency);
      }
    }
    for (const dependency of this.#addedSets[key]) {
      dependencies.add(dependency);
    }
    return Array.from(dependencies);
  }

  private clear(key: DependencyKey) {
    const removed = this.#removedSets[key];
    removed.clear();
    for (const dependency of this.#existingSets[key]) {
      removed.add(dependency);
    }
    this.#addedSets[key].clear();
  }
}
