require("./cjs/assign-exports-property?1").abc;
require("./cjs/assign-exports-property?2");

require("./cjs/assign-module-exports-property?1").abc;
require("./cjs/assign-module-exports-property?2");

require("./cjs/assign-this-property?1").abc;
require("./cjs/assign-this-property?2");

require("./cjs/define-exports-property?1").abc;
require("./cjs/define-exports-property?2");

require("./cjs/define-module-exports-property?1").abc;
require("./cjs/define-module-exports-property?2");

require("./cjs/define-this-property?1").abc;
require("./cjs/define-this-property?2");

require("./cjs/reading-self-from-exports").test;
require("./cjs/reading-self-from-module-exports").test;
require("./cjs/reading-self-from-this").test;

require("./cjs/attach-to-object?1").abc;
require("./cjs/attach-to-object?2").def;
require("./cjs/attach-to-object?3").abc;
require("./cjs/attach-to-object?3").def;

require("./cjs/attach-to-function?1")();
require("./cjs/attach-to-function?2").def;
require("./cjs/attach-to-function?3")();
require("./cjs/attach-to-function?3").def;

require("./cjs/attach-to-arrow-function?1")();
require("./cjs/attach-to-arrow-function?2").def;
require("./cjs/attach-to-arrow-function?3")();
require("./cjs/attach-to-arrow-function?3").def;

require("./cjs/require-default").moduleExportsDefault;
require("./cjs/require-default").hello1;
require("./cjs/require-default").hello2;
require("./cjs/require-default").hello3;
require("./cjs/require-default").hello4;
require("./cjs/require-default").hello5;
require("./cjs/require-default").hello6;
require("./cjs/require-default").hello7;
require("./cjs/require-default").hello8;
