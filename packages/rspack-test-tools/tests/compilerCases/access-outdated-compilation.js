let lastCompilation;

class MyPlugin {
    apply(compiler) {
        compiler.hooks.compilation.tap("Plugin", compilation => {
            if (lastCompilation) {
                return;
            }
            lastCompilation = compilation;
        });
    }
}

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
    description: "throw error when access outdated compilation",
    options(context) {
        return {
            context: context.getSource(),
            entry: "./d",
            plugins: [new MyPlugin()]
        };
    },
    async build(_, compiler) {
        await new Promise(resolve => {
            compiler.run(() => {
                compiler.run(() => {
                    resolve();
                });
            });
        });
    },
    async check() {
        expect(() => lastCompilation.modules).toThrow(/Unable to access compilation/);
    }
};
