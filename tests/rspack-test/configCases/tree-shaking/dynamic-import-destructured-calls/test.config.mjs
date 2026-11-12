export default {
  validate({ stats }) {
    const { modules } = stats[0].toJson({
      all: false,
      modules: true,
      usedExports: true,
    });
    const contexts = modules.filter(module => module.identifier?.includes("|lazy|"));
    expect(contexts).toHaveLength(1);
    expect(contexts[0].identifier).toContain("referencedExports: default|");
    const commonjs = modules.find(module => module.name === "./lib.cjs");
    expect(commonjs.usedExports).toEqual(["missing", "provided"]);
  },
};
