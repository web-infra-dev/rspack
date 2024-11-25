/** @type {import('@rspack/core').Configuration} */
module.exports = {
   module: {
     rules: [
        {
            test: /\.js$/,
            use: [
                {
                    loader: "builtin:swc-loader",
                    options: {
                        jsc: {
                            parser2: {
                                syntax: "ecmascript"
                            }
                        },
                    }
                }
            ]
        }
     ]
   }
};
