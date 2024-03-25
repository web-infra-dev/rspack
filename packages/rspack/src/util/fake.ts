import MergeCaller from "./MergeCaller";

export type FakeHook<T> = T & { _fakeHook: true };

export function createFakeCompilationDependencies(
	getDeps: () => string[],
	addDeps: (deps: string[]) => void
) {
	const addDepsCaller = new MergeCaller(addDeps, 10);
	return {
		*[Symbol.iterator]() {
			const deps = getDeps();
			for (const dep of deps) {
				yield dep;
			}
		},
		has(dep: string): boolean {
			return getDeps().includes(dep);
		},
		add: (dep: string) => {
			addDepsCaller.push(dep);
		},
		addAll: (deps: Iterable<string>) => {
			addDepsCaller.push(...deps);
		}
	};
}
