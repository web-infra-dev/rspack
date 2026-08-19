const consumer = require("./consumer-b.cjs");

process.__commonjsExternalValues.push(consumer);

export const value = consumer.value;
