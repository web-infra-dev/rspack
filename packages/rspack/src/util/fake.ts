import MergeCaller from './MergeCaller';

export type FakeHook<T> = T & { _fakeHook: true };

export function createFakeCompilationDependencies(
  getDeps: () => string[],
  addDeps: (deps: string[]) => void,
) {
  const addDepsCaller = new MergeCaller(addDeps);
  const deletedDeps = new Set<string>();

  const hasDep = (dep: string): boolean => {
    if (deletedDeps.has(dep)) {
      return false;
    }
    return addDepsCaller.pendingData().includes(dep) || getDeps().includes(dep);
  };

  const getAllDeps = () => {
    const deps = new Set([...getDeps(), ...addDepsCaller.pendingData()]);
    for (const deleted of deletedDeps) {
      deps.delete(deleted);
    }
    return deps;
  };

  return {
    *[Symbol.iterator]() {
      const deps = getAllDeps();
      for (const dep of deps) {
        yield dep;
      }
    },
    has: hasDep,
    add: (dep: string) => {
      deletedDeps.delete(dep);
      addDepsCaller.push(dep);
    },
    addAll: (deps: Iterable<string>) => {
      for (const dep of deps) {
        deletedDeps.delete(dep);
      }
      addDepsCaller.push(...deps);
    },
    delete: (dep: string) => {
      const hadDep = hasDep(dep);
      if (hadDep) {
        deletedDeps.add(dep);
      }
      return hadDep;
    },
    keys() {
      return getAllDeps().keys();
    },
    values() {
      return getAllDeps().values();
    },
    entries() {
      return getAllDeps().entries();
    },
    get size() {
      return getAllDeps().size;
    },
  };
}
