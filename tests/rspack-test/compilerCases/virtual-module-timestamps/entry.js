exports.value = require('./virtual/value.js');
const translations = require.context('./translations', false, /\.js$/);
exports.translations = Object.fromEntries(
  translations
    .keys()
    .sort()
    .map((key) => [key, translations(key)]),
);
