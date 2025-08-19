import {value} from './shared'

expect(Object.keys(__STATS__.entrypoints)).toEqual(["bundle0", "bundle1"]);

const index0 = __STATS__.modules.find(m => m.name === "./index0.js");
expect(index0.built).toBe(true);
expect(index0.reasons.length).toBe(1);
expect(index0.reasons[0].type).toBe("entry");

const index1 = __STATS__.modules.find(m => m.name === "./index1.js");
expect(index1.built).toBe(true);
expect(index1.reasons.length).toBe(1);
expect(index1.reasons[0].type).toBe("entry");

const shared = __STATS__.modules.find(m => m.name === "./shared.js");
expect(value).toBe(42)
expect(shared.built).toBe(true);

expect(shared.reasons.length).toBe(4);
