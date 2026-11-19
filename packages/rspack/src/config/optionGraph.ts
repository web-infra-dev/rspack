import binding, { type RawConfigValue } from '@rspack/binding';

interface Shape {
  fields?: string[];
  children?: Record<string, Shape>;
  values?: Shape;
  elements?: Shape;
  booleansOnly?: boolean;
}

const leaf: Shape = {};
const regexp: Shape = { fields: ['source', 'flags'] };
const booleanMap: Shape = { booleansOnly: true };
let fields: Record<string, string[]> | undefined;
const shapes = new Map<string, Shape>();

function getFields() {
  return (fields ??= binding.getConfigValueFields());
}

function getShape(name: string): Shape {
  const existing = shapes.get(name);
  if (existing) return existing;

  const shape: Shape = { fields: getFields()[name] };
  shapes.set(name, shape);
  if (name.startsWith('parser:')) {
    shape.children = {
      wrappedContextRegExp: regexp,
      importMeta: booleanMap,
      worker: getShape('worker'),
      commonjs: getShape('commonjs'),
      dataUrlCondition: getShape('dataUrlCondition'),
    };
  } else if (name.startsWith('generator:')) {
    shape.children = { dataUrl: getShape('dataUrl') };
  } else if (name === 'resolve') {
    shape.children = {
      alias: getShape('aliasArray'),
      fallback: getShape('aliasArray'),
      tsconfig: getShape('tsconfig'),
      byDependency: getShape('byDependency'),
      extensionAlias: { values: leaf },
      restrictions: { elements: regexp },
    };
  } else if (name === 'byDependency') {
    shape.values = getShape('resolve');
  } else if (name === 'aliasArray') {
    shape.elements = getShape('alias');
  }
  return shape;
}

type Reference = [number];
type Node = Record<string, unknown> | unknown[];

interface Entry {
  reference: Reference;
  node: Node;
  shapes: Set<Shape>;
  values: Map<string | number, unknown>;
}

export function encodeConfigDom(
  source: Record<string, unknown>,
  schema: string,
): RawConfigValue {
  const nodes: Node[] = [];
  const seen = new WeakMap<object, Entry>();

  function encode(value: unknown, shape: Shape): unknown {
    if (typeof value !== 'object' || value === null) return value;

    let entry = seen.get(value);
    if (entry === undefined) {
      const node: Node = Array.isArray(value) ? [] : Object.create(null);
      entry = {
        reference: [nodes.length],
        node,
        shapes: new Set(),
        values: new Map(),
      };
      seen.set(value, entry);
      nodes.push(node);
    }
    if (entry.shapes.has(shape)) return entry.reference;
    entry.shapes.add(shape);

    // The same object can be used in several option shapes. Read each field
    // once and record the union of the fields selected by those shapes.
    const { node, values } = entry;
    if (Array.isArray(node)) {
      const array = value as unknown[];
      for (let index = 0; index < array.length; index++) {
        if (!values.has(index)) values.set(index, array[index]);
        node[index] = encode(values.get(index), shape.elements ?? leaf);
      }
    } else {
      const keys =
        shape.values || shape.booleansOnly
          ? Object.keys(value)
          : (shape.fields ?? []);
      const object = value as Record<string, unknown>;
      for (const key of keys) {
        if (!values.has(key)) values.set(key, object[key]);
        const item = values.get(key);
        if (
          item === undefined ||
          (shape.booleansOnly && typeof item !== 'boolean')
        ) {
          continue;
        }
        node[key] = encode(
          item,
          (shape.children && Object.hasOwn(shape.children, key)
            ? shape.children[key]
            : shape.values) ?? leaf,
        );
      }
    }
    return entry.reference;
  }

  const [root] = encode(source, getShape(schema)) as Reference;
  return { root, nodes };
}

export function encodeModuleOptions(
  kind: 'parser' | 'generator',
  options: Record<string, Record<string, unknown>>,
  adapt?: (
    type: string,
    value: Record<string, unknown>,
  ) => Record<string, unknown>,
): Record<string, RawConfigValue> {
  const fields = getFields();
  return Object.fromEntries(
    Object.keys(options).flatMap<[string, RawConfigValue]>((type) => {
      const schema = `${kind}:${type}`;
      if (!Object.hasOwn(fields, schema)) {
        // Rust handles ignored and unknown module types.
        return [[type, { root: 0, nodes: [{}] }]];
      }
      const value = options[type];
      return value == null
        ? []
        : [[type, encodeConfigDom(adapt?.(type, value) ?? value, schema)]];
    }),
  );
}
