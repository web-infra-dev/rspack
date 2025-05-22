/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry: "./index",
    plugins: [
        {
            apply(compiler) {
                compiler.hooks.done.tap("TestPlugin", stats => {
                    const erros = stats.compilation.errors;
                    expect(erros.length).toBe(1);
                    expect(erros[0].name).toBe("ModuleNotFoundError");
                    expect(erros[0].message).toContain("Module not found: Can't resolve './index'");

                    expect(erros[0].error).toBeDefined();
                });
            }
        }
    ]
};
