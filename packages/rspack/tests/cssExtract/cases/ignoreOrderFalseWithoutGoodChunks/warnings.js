const cssLoaderPath = require.resolve("css-loader").replace(/\\/g, "/");

module.exports = `WARNING in ⚠ chunk styles [rspack-mini-css-extract-plugin]
  │ Conflicting order. Following module has been added:
  │  * css ${cssLoaderPath}!./e1.css
  │ despite it was not able to fulfill desired ordering with these modules:
  │  * css ${cssLoaderPath}!./e2.css
  │   - couldn't fulfill desired order of chunk group(s) entry2

WARNING in ⚠ chunk styles [rspack-mini-css-extract-plugin]
  │ Conflicting order. Following module has been added:
  │  * css ${cssLoaderPath}!./e4.css
  │ despite it was not able to fulfill desired ordering with these modules:
  │  * css ${cssLoaderPath}!./e3.css
  │   - couldn't fulfill desired order of chunk group(s) entry3
  │   - while fulfilling desired order of chunk group(s) entry4

WARNING in ⚠ chunk styles [rspack-mini-css-extract-plugin]
  │ Conflicting order. Following module has been added:
  │  * css ${cssLoaderPath}!./e2.css
  │ despite it was not able to fulfill desired ordering with these modules:
  │  * css ${cssLoaderPath}!./e3.css
  │   - couldn't fulfill desired order of chunk group(s) entry3
  │   - while fulfilling desired order of chunk group(s) entry4`;
