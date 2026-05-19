const lazyModules = import.meta.glob("./dir/*.js");
const eagerModules = import.meta.glob("./dir/*.js", { eager: true });

console.log(Object.keys(lazyModules).sort());
console.log(Object.keys(eagerModules).sort());

export { eagerModules, lazyModules };
