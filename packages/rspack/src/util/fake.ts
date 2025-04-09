import MergeCaller from "./MergeCaller";

export type FakeHook<T> = T & { _fakeHook: true };

export function createFakeCompilationDependencies(
	getDeps: () => string[],
	addDeps: (deps: string[]) => void
) {
	const addDepsCaller = new MergeCaller(addDeps);
	return {
		*[Symbol.iterator]() {
			const deps = new Set([...getDeps(), ...addDepsCaller.pendingData()]);
			for (const dep of deps) {
				yield dep;
			}
		},
		has(dep: string): boolean {
			return (
				addDepsCaller.pendingData().includes(dep) || getDeps().includes(dep)
			);
		},
		add: (dep: string) => {
			addDepsCaller.push(dep);
		},
		addAll: (deps: Iterable<string>) => {
			addDepsCaller.push(...deps);
		}
	};
}
