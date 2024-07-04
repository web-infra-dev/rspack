// eslint-disable-next-line node/no-unpublished-require
const CLI = require("../../packages/webpack-cli/lib/webpack-cli");

describe("CLI API", () => {
  let cli;

  beforeEach(() => {
    cli = new CLI();
  });

  describe("makeCommand", () => {
    it("should make command", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand({ name: "command" }, [], (options) => {
        expect(options).toEqual({});
      });

      command.parseAsync([], { from: "user" });
    });

    it("should make command with Boolean option by default", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: true });
        },
      );

      command.parseAsync(["--boolean"], { from: "user" });
    });

    it("should make command with Boolean option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: Boolean,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: true });
        },
      );

      command.parseAsync(["--boolean"], { from: "user" });
    });

    it("should make command with Boolean option and negative value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: Boolean,
            description: "description",
            negative: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: false });
        },
      );

      command.parseAsync(["--no-boolean"], { from: "user" });
    });

    it("should make command with configs boolean option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "configs-boolean",
            configs: [
              {
                type: "boolean",
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ configsBoolean: false });
        },
      );

      command.parseAsync(["--no-configs-boolean"], { from: "user" });
    });

    it("should make command with configs number option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "configs-number",
            configs: [
              {
                type: "number",
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ configsNumber: 42 });
        },
      );

      command.parseAsync(["--configs-number", "42"], { from: "user" });
    });

    it("should make command with configs string option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "configs-string",
            configs: [
              {
                type: "string",
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ configsString: "foo" });
        },
      );

      command.parseAsync(["--configs-string", "foo"], { from: "user" });
    });

    it("should make command with configs path option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "configs-path",
            configs: [
              {
                type: "path",
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ configsPath: "/root/foo" });
        },
      );

      command.parseAsync(["--configs-path", "/root/foo"], {
        from: "user",
      });
    });

    it("should make command with configs RegExp option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "configs-regexp",
            configs: [
              {
                type: "RegExp",
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ configsRegexp: "\\w+" });
        },
      );

      command.parseAsync(["--configs-regexp", "\\w+"], { from: "user" });
    });

    it("should make command with configs enum/string option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "enum-string",
            configs: [
              {
                type: "enum",
                values: ["foo"],
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ enumString: "foo" });
        },
      );

      command.parseAsync(["--enum-string", "foo"], { from: "user" });
    });

    it("should make command with configs enum/number option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "enum-number",
            configs: [
              {
                type: "enum",
                values: [42],
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ enumNumber: 42 });
        },
      );

      command.parseAsync(["--enum-number", "42"], { from: "user" });
    });

    it("should make command with configs enum/boolean option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "enum-boolean",
            configs: [
              {
                type: "boolean",
                values: [false],
              },
            ],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ enumBoolean: false });
        },
      );

      command.parseAsync(["--no-enum-boolean"], { from: "user" });
    });

    it("should make command with Boolean option and negative value #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: Boolean,
            description: "description",
            negative: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: false });
        },
      );

      command.parseAsync(["--boolean", "--no-boolean"], { from: "user" });
    });

    it("should make command with Boolean option and negative value #3", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: Boolean,
            description: "description",
            negative: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: true });
        },
      );

      command.parseAsync(["--no-boolean", "--boolean"], { from: "user" });
    });

    it("should make command with Boolean option with default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: Boolean,
            description: "description",
            defaultValue: false,
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: false });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with String option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            type: String,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: "bar" });
        },
      );

      command.parseAsync(["--string", "bar"], { from: "user" });
    });

    it("should make command with String option with alias", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            alias: "s",
            type: String,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: "foo" });
        },
      );

      command.parseAsync(["-s", "foo"], { from: "user" });
    });

    it("should make command with String option with default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            type: String,
            description: "description",
            defaultValue: "default-value",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: "default-value" });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with String option with default value #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            type: String,
            description: "description",
            defaultValue: "default-value",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: "foo" });
        },
      );

      command.parseAsync(["--string", "foo"], { from: "user" });
    });

    it('should make command with String option using "=" syntax', async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            type: String,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: "bar" });
        },
      );

      command.parseAsync(["--string=bar"], { from: "user" });
    });

    it("should make command with multiple String option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            multiple: true,
            type: String,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: ["foo", "bar"] });
        },
      );

      command.parseAsync(["--string", "foo", "bar"], { from: "user" });
    });

    it("should make command with multiple String option with default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            multiple: true,
            type: String,
            description: "description",
            defaultValue: "string",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: "string" });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with multiple String option with default value #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            multiple: true,
            type: String,
            description: "description",
            defaultValue: "string",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: ["foo", "bar"] });
        },
      );

      command.parseAsync(["--string", "foo", "--string", "bar"], {
        from: "user",
      });
    });

    it("should make command with multiple String option #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "string",
            multiple: true,
            type: String,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ string: ["foo", "bar"] });
        },
      );

      command.parseAsync(["--string", "foo", "--string", "bar"], {
        from: "user",
      });
    });

    it("should make command with Number option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "number",
            type: Number,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ number: 12 });
        },
      );

      command.parseAsync(["--number", "12"], { from: "user" });
    });

    it("should make command with Number option with default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "number",
            type: Number,
            description: "description",
            defaultValue: 20,
          },
        ],
        (options) => {
          expect(options).toEqual({ number: 20 });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with multiple Number option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "number",
            multiple: true,
            type: Number,
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ number: [1, 2] });
        },
      );

      command.parseAsync(["--number", "1", "--number", "2"], {
        from: "user",
      });
    });

    it("should make command with multiple Number option and default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "number",
            multiple: true,
            type: Number,
            description: "description",
            defaultValue: 50,
          },
        ],
        (options) => {
          expect(options).toEqual({ number: [1, 2] });
        },
      );

      command.parseAsync(["--number", "1", "--number", "2"], {
        from: "user",
      });
    });

    it("should make command with multiple Number option and default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "number",
            multiple: true,
            type: Number,
            description: "description",
            defaultValue: 50,
          },
        ],
        (options) => {
          expect(options).toEqual({ number: 50 });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with custom function type", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "custom",
            type: () => {
              return "function";
            },
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ custom: "function" });
        },
      );

      command.parseAsync(["--custom", "value"], { from: "user" });
    });

    it("should make command with custom function type and default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "custom",
            type: () => {
              return "function";
            },
            description: "description",
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({ custom: "default" });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with multiple custom function type", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "custom",
            type: (value, previous = []) => {
              return previous.concat([value]);
            },
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ custom: ["value", "other"] });
        },
      );

      command.parseAsync(["--custom", "value", "--custom", "other"], {
        from: "user",
      });
    });

    it("should make command with multiple custom function type and default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "custom",
            type: (value, previous = []) => {
              return previous.concat([value]);
            },
            description: "description",
            multiple: true,
            defaultValue: 50,
          },
        ],
        (options) => {
          expect(options).toEqual({ custom: 50 });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with multiple custom function type and default value #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      let skipDefault = true;

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "custom",
            type: (value, previous = []) => {
              if (skipDefault) {
                previous = [];
                skipDefault = false;
              }

              return [].concat(previous).concat([value]);
            },
            description: "description",
            multiple: true,
            defaultValue: 50,
          },
        ],
        (options) => {
          expect(options).toEqual({ custom: ["foo"] });
        },
      );

      command.parseAsync(["--custom", "foo"], { from: "user" });
    });

    it("should make command with Boolean and String option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-string",
            type: [Boolean, String],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndString: true });
        },
      );

      command.parseAsync(["--boolean-and-string"], { from: "user" });
    });

    it("should make command with Boolean and String option #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-string",
            type: [Boolean, String],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndString: "value" });
        },
      );

      command.parseAsync(["--boolean-and-string", "value"], {
        from: "user",
      });
    });

    it("should make command with multiple Boolean and String option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-string",
            type: [Boolean, String],
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndString: true });
        },
      );

      command.parseAsync(["--boolean-and-string"], { from: "user" });
    });

    it("should make command with multiple Boolean and String option #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-string",
            type: [Boolean, String],
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndString: ["bar", "baz"],
          });
        },
      );

      command.parseAsync(["--boolean-and-string", "bar", "--boolean-and-string", "baz"], {
        from: "user",
      });
    });

    it("should make command with Boolean and String option and negative", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-string",
            type: [Boolean, String],
            description: "description",
            negative: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndString: true });
        },
      );

      command.parseAsync(["--boolean-and-string"], { from: "user" });
    });

    it("should make command with Boolean and String option and negative #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-string",
            type: [Boolean, String],
            description: "description",
            negative: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndString: "foo" });
        },
      );

      command.parseAsync(["--boolean-and-string", "foo"], {
        from: "user",
      });
    });

    it("should make command with Boolean and String option and negative #3", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-string",
            type: [Boolean, String],
            description: "description",
            negative: true,
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndString: false });
        },
      );

      command.parseAsync(["--no-boolean-and-string"], { from: "user" });
    });

    it("should make command with Boolean and Number option", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number",
            type: [Boolean, Number],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndNumber: true });
        },
      );

      command.parseAsync(["--boolean-and-number"], { from: "user" });
    });

    it("should make command with Boolean and Number option #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number",
            type: [Boolean, Number],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndNumber: 12 });
        },
      );

      command.parseAsync(["--boolean-and-number", "12"], {
        from: "user",
      });
    });

    it("should make command with array Boolean type", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: [Boolean],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: true });
        },
      );

      command.parseAsync(["--boolean"], { from: "user" });
    });

    it("should make command with Boolean and Number and String type", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: true,
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string"], {
        from: "user",
      });
    });

    it("should make command with Boolean and Number and String type #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndNumberAndString: 12 });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "12"], {
        from: "user",
      });
    });

    it("should make command with Boolean and Number and String type #3", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: "bar",
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "bar"], {
        from: "user",
      });
    });

    it("should make command with Boolean and Number and String type and default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: "default",
          });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with Boolean and Number and String type and default value #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: "foo",
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "foo"], {
        from: "user",
      });
    });

    it("should make command with Boolean and Number and String type and default value #3", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({ booleanAndNumberAndString: 12 });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "12"], {
        from: "user",
      });
    });

    it("should make command with Boolean and Number and String type and default value #4", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: true,
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string"], {
        from: "user",
      });
    });

    it("should make command with multiple Boolean and Number and String type", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: true,
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string"], {
        from: "user",
      });
    });

    it("should make command with multiple Boolean and Number and String type #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: ["foo"],
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "foo"], {
        from: "user",
      });
    });

    it("should make command with multiple Boolean and Number and String type #3", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: [12],
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "12"], {
        from: "user",
      });
    });

    it("should make command with multiple Boolean and Number and String type #4", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: ["foo", "bar"],
          });
        },
      );

      command.parseAsync(
        ["--boolean-and-number-and-string", "foo", "--boolean-and-number-and-string", "bar"],
        { from: "user" },
      );
    });

    it("should make command with multiple Boolean and Number and String type #5", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: ["foo", 12],
          });
        },
      );

      command.parseAsync(
        ["--boolean-and-number-and-string", "foo", "--boolean-and-number-and-string", "12"],
        { from: "user" },
      );
    });

    it("should make command with multiple Boolean and Number and String and default value", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: "default",
          });
        },
      );

      command.parseAsync([], { from: "user" });
    });

    it("should make command with multiple Boolean and Number and String and default value #2", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: ["foo"],
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "foo"], {
        from: "user",
      });
    });

    it("should make command with multiple Boolean and Number and String and default value #3", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: [12],
          });
        },
      );

      command.parseAsync(["--boolean-and-number-and-string", "12"], {
        from: "user",
      });
    });

    it("should make command with multiple Boolean and Number and String and default value #4", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean-and-number-and-string",
            type: [Boolean, Number, String],
            description: "description",
            multiple: true,
            defaultValue: "default",
          },
        ],
        (options) => {
          expect(options).toEqual({
            booleanAndNumberAndString: ["foo", 12],
          });
        },
      );

      command.parseAsync(
        ["--boolean-and-number-and-string", "foo", "--boolean-and-number-and-string", "12"],
        { from: "user" },
      );
    });

    it("should make command with array of unknown types", async () => {
      expect.assertions(1);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "unknown",
            type: [Boolean, Symbol],
            description: "description",
          },
        ],
        (options) => {
          expect(options).toEqual({ unknown: "foo" });
        },
      );

      command.parseAsync(["--unknown", "foo"], { from: "user" });
    });

    it("should make command with Boolean option and use description", async () => {
      expect.assertions(2);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: Boolean,
            description: "Description",
            negatedDescription: "Negated description",
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: true });
        },
      );

      command.parseAsync(["--boolean"], { from: "user" });

      expect(command.helpInformation()).toContain("--boolean   Description");
    });

    it("should make command with Boolean option and negative value and use negatedDescription", async () => {
      expect.assertions(2);

      cli.program.commands = [];

      const command = await cli.makeCommand(
        {
          name: "command",
        },
        [
          {
            name: "boolean",
            type: Boolean,
            description: "description",
            negative: true,
            negatedDescription: "Negated description",
          },
        ],
        (options) => {
          expect(options).toEqual({ boolean: false });
        },
      );

      command.parseAsync(["--no-boolean"], { from: "user" });

      expect(command.helpInformation()).toContain("--no-boolean  Negated description");
    });
  });

  describe("custom help output", () => {
    let consoleSpy;
    let exitSpy;

    beforeEach(async () => {
      consoleSpy = jest.spyOn(global.console, "log");
      exitSpy = jest.spyOn(process, "exit").mockImplementation(() => {});

      cli.program.option("--color [value]", "any color", "blue");

      await new Promise((resolve, reject) => {
        try {
          cli.run(["help", "--color"], { from: "user" });
          resolve();
        } catch (error) {
          reject(error);
        }
      });
    });

    afterEach(async () => {
      consoleSpy.mockRestore();
      exitSpy.mockRestore();
    });

    it("should display help information", () => {
      expect(exitSpy).toHaveBeenCalledWith(0);
      expect(consoleSpy.mock.calls).toMatchSnapshot();
    });
  });
});
