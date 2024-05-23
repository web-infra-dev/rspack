declare var global: {
  updateSnapshot: boolean;
};

export function isUpdateSnapshot() {
  return global.updateSnapshot || process.env.UPDATE_SNAPSHOT === "true"
}