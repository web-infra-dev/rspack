class MyPlugin {
    apply(compiler) {
        compiler.hooks.compilation.tap("MyPlugin", compilation => {
            compilation.hooks.optimizeChunks.tap("MyPlugin", chunks => {
                chunks = [...chunks]
                expect(chunks.length).toEqual(1);
                expect(chunks[0].name).toBe("main");
            });
        });
    }
}

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
    description: "should call optimizeChunks hook correctly",
    options(context) {
        return {
            context: context.getSource(),
            entry: "./d",
            plugins: [new MyPlugin()]
        };
    }
};
