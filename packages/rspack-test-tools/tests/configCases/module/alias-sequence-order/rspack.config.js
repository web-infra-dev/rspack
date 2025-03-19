/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
    module: {
        rules: [
            {
                resolve: {
                    alias: {
                        "foo/bar": "./exist"
                    }
                },
            },
            {
                resolve: {
                    alias: {
                        "foo": "./not-exist"
                    }
                },
            },
        ]
    },
};
