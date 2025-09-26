const { basename, defineCompileCase } = require("@rspack/test-tools");
const customFieldValues = [];

class MyPlugin {
    apply(compiler) {
        compiler.hooks.compilation.tap("Plugin", compilation => {
            compilation.hooks.finishModules.tap("Plugin", modules => {
                for (const module of modules) {
                    customFieldValues.push(module.buildInfo.foo);
                }
            });
        });
    }
}


defineCompileCase(Utils.basename(__filename), {
    description: "should persist build info custom fields",
    options(context) {
        return {
            context: context.getSource(),
            entry: "./d",
            plugins: [new MyPlugin()],
            cache: true,
            experiments: {
                cache: {
                    type: "persistent"
                }
            },
            module: {
                rules: [
                    {
                        test: /\.js$/,
                        use: [
                            {
                                loader: context.getSource("build-info-loader.js"),
                            }
                        ]
                    }
                ]
            }
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
        expect(customFieldValues.length).toBeGreaterThan(0);
        customFieldValues.forEach(foo => {
            expect(foo).toBeTruthy();
        });
    }
});
