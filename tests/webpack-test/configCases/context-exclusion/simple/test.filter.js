// rspack don't have ContextExclusionPlugin.
// skip it, because ContextExclusionPlugin already instead of IgnorePlugin.
module.exports = () => { return false }
