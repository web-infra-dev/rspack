import { loaders } from "./barrel";

expect(globalThis.unusedObjectInitializerRuns).toEqual(1);
expect(loaders).toEqual([]);
