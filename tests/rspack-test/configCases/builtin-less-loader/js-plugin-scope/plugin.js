module.exports = {
  install(less, manager, functions) {
    const saved = [];
    function table(context, name) {
      for (const frame of context.frames) {
        const vars = frame.variables();
        if (Object.prototype.hasOwnProperty.call(vars, name)) return vars;
      }
      throw new Error('missing variable ' + name);
    }
    functions.add('scope-read', function (name) {
      return table(this.context, name.value)[name.value].value.eval(this.context);
    });
    functions.add('scope-save', function () {
      saved.push(table(this.context, '@scope-value'));
      return new less.tree.Dimension(saved.length);
    });
    functions.add('scope-saved', function (index) {
      return saved[index.value]['@scope-value'].value.eval(this.context);
    });
    functions.add('scope-inspect', function () {
      const vars = table(this.context, '@scope-first');
      const names = Object.keys(vars).filter(name => name.startsWith('@scope-')).sort();
      const first = Object.getOwnPropertyDescriptor(vars, '@scope-first');
      if (names.join(',') !== '@scope-first,@scope-second' || !first.enumerable || !first.configurable || !first.writable) throw new Error('variable descriptors');
      if (vars['@absent'] !== undefined || '@absent' in vars) throw new Error('missing variable');
      const copy = Object.assign({}, vars);
      if (copy['@scope-first'] !== vars['@scope-first']) throw new Error('variable identity');
      const second = vars['@scope-second'];
      delete vars['@scope-second'];
      if ('@scope-second' in vars || Object.keys(vars).includes('@scope-second')) throw new Error('delete variable');
      vars['@scope-second'] = second;
      Object.defineProperty(vars, '@scope-first', { value: second });
      if (vars['@scope-first'] !== second) throw new Error('replace variable');
      Object.freeze(vars);
      if (!Object.isFrozen(vars) || vars['@scope-second'] !== second) throw new Error('freeze variables');
      return new less.tree.Keyword('ok');
    });
  }
};
