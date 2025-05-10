const {
  Confirm,
  List,
  Input,
  // eslint-disable-next-line node/no-missing-require
} = require("../../../packages/generators/src/utils/scaffold-utils");

describe("utils", () => {
  let mockSelf;

  beforeEach(() => {
    mockSelf = {
      prompt: (arg) => {
        return arg[0];
      },
    };
  });
  describe("Inquirer", () => {
    it("should emulate a prompt for List", () => {
      expect(List(mockSelf, "entry", "does it work?", ["Yes", "Maybe"], "Yes")).toEqual({
        choices: ["Yes", "Maybe"],
        type: "list",
        name: "entry",
        message: "does it work?",
        default: "Yes",
      });
    });

    it("should make default value for a List", () => {
      expect(List(mockSelf, "entry", "does it work?", ["Yes", "Maybe"], "Yes", true)).toEqual({
        entry: "Yes",
      });
    });

    it("should emulate a prompt for list input", () => {
      expect(Input(mockSelf, "plugins", "what is your plugin?", "openJSF")).toEqual({
        type: "input",
        name: "plugins",
        message: "what is your plugin?",
        default: "openJSF",
      });
    });

    it("should return a default Input object value", () => {
      expect(Input(mockSelf, "plugins", "what is your plugin?", "my-plugin", true)).toEqual({
        plugins: "my-plugin",
      });
    });

    it("should emulate a prompt for confirm", () => {
      expect(Confirm(mockSelf, "context", "what is your context?")).toEqual({
        name: "context",
        default: true,
        message: "what is your context?",
        type: "confirm",
      });
    });

    it("should make a Confirm object with yes as default", () => {
      expect(Confirm(mockSelf, "context", "what is your context?", true, true)).toEqual({
        context: true,
      });
    });
  });
});
