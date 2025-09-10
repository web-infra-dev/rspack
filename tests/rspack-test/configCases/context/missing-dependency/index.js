const lang = ["en", "fr", "es", "de", "it", "ja", "zh", "ru", "pt", "ar"];

for (const l of lang) {
  require(`./lang/${l}`);
}

