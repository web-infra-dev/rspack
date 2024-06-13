const x = {};
Object.defineProperty(x, "named", { value: "named" });
Object.defineProperty(x, "default", { value: "default" });
Object.setPrototypeOf(exports, x);
