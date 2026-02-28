const loadLocale = (name) => {
  let locale;
  try {
    locale = require('./locale/' + name);
  } catch (e) {
  }
  return locale;
};

module.exports = {
  loadLocale,
}