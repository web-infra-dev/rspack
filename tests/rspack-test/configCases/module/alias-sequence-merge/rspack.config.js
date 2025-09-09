/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
    module: {
        rules: [
            {
                resolve: {
                    alias: {
                        "foo": "./not-exist"
                    }
                },
            },
            {
                resolve: {
                    alias: {
                        "foo": "./exist"
                    }
                },
            },
        ]
    },
};
