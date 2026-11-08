import * as foo from "foo";
import * as file1 from "foo/file1";

import * as bar from "bar/file1";
import * as myStar from "star-bar";
import * as longest from "longest/bar";
import * as packagedBrowser from "browser-field-package";
import * as packagedMain from "main-field-package";
import * as packagedIndex from "no-main-field-package";
import * as newFile from "utils/old-file";

console.log(
  "HELLO WORLD!",
  foo.message,
  bar.message,
  file1,
  longest,
  myStar.message,
  packagedBrowser.message,
  packagedMain.message,
  packagedIndex.message,
  newFile
);
