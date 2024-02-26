module.exports = `WARNING in ⚠ chunk styles [rspack-mini-css-extract-plugin]
  │ Conflicting order. Following module has been added:
  │  * css /path/to/loader.js!./e2.css
  │ despite it was not able to fulfill desired ordering with these modules:
  │  * css /path/to/loader.js!./e1.css
  │   - couldn't fulfill desired order of chunk group(s) entry2
  │   - while fulfilling desired order of chunk group(s) entry1`;
