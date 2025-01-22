/** @type {import("../../../..").THookCaseConfig} */
let callOrder = [];
module.exports = {
    description: "should call js hooks correctly",
    options(context) {

        return {
            plugins: [
                {
                    apply(compiler) {
                        compiler.hooks.emit.tap({
                                name: "normal",
                                stage: 42,
                            }, () => {
                                callOrder.push("42")
                            }
                        );

                        compiler.hooks.emit.tap({
                                name: "Positive Infinity",
                                stage: Number.POSITIVE_INFINITY
                            }, () => {
                                callOrder.push("positive")
                            }
                        );

                        compiler.hooks.emit.tap({
                                name: "Negative Infinity",
                                stage: Number.NEGATIVE_INFINITY
                            }, () => {
                                callOrder.push("negative")
                            }
                        );
                    }
                }
            ]
        };
    },
    async check() {
        expect(callOrder).toStrictEqual([
            "negative", "42", "positive",
        ])
    }
};
