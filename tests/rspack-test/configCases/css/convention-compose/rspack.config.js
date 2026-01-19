const path = require("path");
module.exports = {
  output: {
    filename: "main.js",
    library: {
      type: "commonjs-static",
    },
  },
  module: {
    generator: {
      "css/auto": {
        exportsConvention: "camel-case",
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: "css/auto"
      }
    ]
  },

  plugins: [{
    apply(compiler) {
      compiler.hooks.done.tap("Test", stats => {
        const { styles } = require(path.join(compiler.options.output.path, "main.js"));
        expect(typeof styles["first-class"]).toBe("string");
        expect(styles["first-class"].split(" ").length).toBe(1);
        expect(typeof styles.firstClass).toBe("string");
        expect(styles.firstClass.split(" ").length).toBe(1);

        expect(typeof styles["second-class"]).toBe("string");
        expect(styles["second-class"].split(" ").length).toBe(2);
        expect(typeof styles.secondClass).toBe("string");
        expect(styles.secondClass.split(" ").length).toBe(2);
      });
    }
  }]
};
