import a from "./a";
await new Promise(r => setTimeout(() => r(a), 100));
