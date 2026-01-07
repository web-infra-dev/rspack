import fs from 'node:fs';
import path from 'node:path';
import {
  BuiltinPluginName,
  type JsHtmlPluginTag,
  type RawHtmlRspackPluginOptions,
} from '@rspack/binding';

import type { Compilation } from '../../Compilation';
import type { Compiler } from '../../Compiler';
import { create } from '../base';
import {
  cleanPluginHooks,
  getPluginHooks,
  type HtmlRspackPluginHooks,
} from './hooks';
import {
  cleanPluginOptions,
  type HtmlRspackPluginOptions,
  setPluginOptions,
} from './options';

type HtmlPluginTag = {
  tagName: string;
  attributes: Record<string, string>;
  voidTag: boolean;
  innerHTML?: string;
  toString?: () => string;
};

let HTML_PLUGIN_UID = 0;

const HtmlRspackPluginImpl = create(
  BuiltinPluginName.HtmlRspackPlugin,
  function (
    this: Compiler,
    c: HtmlRspackPluginOptions = {},
  ): RawHtmlRspackPluginOptions {
    const uid = HTML_PLUGIN_UID++;
    const meta: Record<string, Record<string, string>> = {};
    for (const key in c.meta) {
      const value = c.meta[key];
      if (typeof value === 'string') {
        meta[key] = {
          name: key,
          content: value,
        };
      } else {
        meta[key] = {
          name: key,
          ...value,
        };
      }
    }
    const scriptLoading = c.scriptLoading ?? 'defer';
    const configInject = c.inject ?? true;
    const inject =
      configInject === true
        ? scriptLoading === 'blocking'
          ? 'body'
          : 'head'
        : configInject === false
          ? 'false'
          : configInject;
    const base = typeof c.base === 'string' ? { href: c.base } : c.base;
    const chunksSortMode = c.chunksSortMode ?? 'auto';

    let compilation: Compilation | null = null;
    this.hooks.compilation.tap('HtmlRspackPlugin', (compilationInstance) => {
      compilation = compilationInstance;
      setPluginOptions(compilation, uid, c);
    });
    this.hooks.done.tap('HtmlRspackPlugin', (stats) => {
      cleanPluginHooks(stats.compilation);
      cleanPluginOptions(stats.compilation, uid);
    });

    function generateRenderData(data: string): Record<string, unknown> {
      const json = JSON.parse(data);
      if (typeof c.templateParameters !== 'function') {
        json.compilation = compilation;
      }
      const renderTag = function (this: HtmlPluginTag) {
        return htmlTagObjectToString(this);
      };
      const renderTagList = function (this: HtmlPluginTag[]) {
        return this.join('');
      };
      if (Array.isArray(json.htmlRspackPlugin?.tags?.headTags)) {
        for (const tag of json.htmlRspackPlugin.tags.headTags) {
          tag.toString = renderTag;
        }
        json.htmlRspackPlugin.tags.headTags.toString = renderTagList;
      }
      if (Array.isArray(json.htmlRspackPlugin?.tags?.bodyTags)) {
        for (const tag of json.htmlRspackPlugin.tags.bodyTags) {
          tag.toString = renderTag;
        }
        json.htmlRspackPlugin.tags.bodyTags.toString = renderTagList;
      }
      return json;
    }

    let templateContent = c.templateContent;
    let templateFn: ((data: string) => Promise<string>) | undefined;
    if (typeof templateContent === 'function') {
      templateFn = async (data: string) => {
        try {
          const renderer = c.templateContent as (
            data: Record<string, unknown>,
          ) => Promise<string> | string;
          if (c.templateParameters === false) {
            return await renderer({});
          }
          return await renderer(generateRenderData(data));
        } catch (e) {
          const error = new Error(
            `HtmlRspackPlugin: render template function failed, ${(e as Error).message}`,
          );
          error.stack = (e as Error).stack;
          throw error;
        }
      };
      templateContent = '';
    } else if (c.template) {
      const filename = c.template.split('?')[0];
      if (['.js', '.cjs'].includes(path.extname(filename))) {
        templateFn = async (data: string) => {
          const context = this.options.context || process.cwd();
          const templateFilePath = path.resolve(context, filename);
          if (!fs.existsSync(templateFilePath)) {
            throw new Error(
              `HtmlRspackPlugin: could not load file \`${filename}\` from \`${context}\``,
            );
          }
          try {
            const renderer = (
              IS_BROWSER
                ? this.__internal_browser_require(templateFilePath)
                : require(templateFilePath)
            ) as (data: Record<string, unknown>) => Promise<string> | string;
            if (c.templateParameters === false) {
              return await renderer({});
            }
            return await renderer(generateRenderData(data));
          } catch (e) {
            const error = new Error(
              `HtmlRspackPlugin: render template function failed, ${(e as Error).message}`,
            );
            error.stack = (e as Error).stack;
            throw error;
          }
        };
      }
    }

    const rawTemplateParameters = c.templateParameters;
    let templateParameters:
      | boolean
      | Record<string, any>
      | ((params: string) => Promise<string>)
      | undefined;
    if (typeof rawTemplateParameters === 'function') {
      templateParameters = async (data: string) => {
        const newData = await rawTemplateParameters(JSON.parse(data));
        return JSON.stringify(newData);
      };
    } else {
      templateParameters = rawTemplateParameters;
    }

    let filenames: Set<string> | undefined;
    if (typeof c.filename === 'string') {
      filenames = new Set();
      if (c.filename.includes('[name]')) {
        if (typeof this.options.entry === 'object') {
          for (const entryName of Object.keys(this.options.entry)) {
            filenames.add(c.filename.replace(/\[name\]/g, entryName));
          }
        } else {
          throw new Error(
            'HtmlRspackPlugin: filename with `[name]` does not support function entry',
          );
        }
      } else {
        filenames.add(c.filename);
      }
    } else if (typeof c.filename === 'function') {
      filenames = new Set();
      if (typeof this.options.entry === 'object') {
        for (const entryName of Object.keys(this.options.entry)) {
          filenames.add(c.filename(entryName));
        }
      } else {
        throw new Error(
          'HtmlRspackPlugin: function filename does not support function entry',
        );
      }
    }

    return {
      filename: filenames ? Array.from(filenames) : undefined,
      template: c.template,
      hash: c.hash,
      title: c.title,
      favicon: c.favicon,
      publicPath: c.publicPath,
      chunks: c.chunks,
      excludeChunks: c.excludeChunks,
      chunksSortMode,
      minify: c.minify,
      meta,
      scriptLoading,
      inject,
      base,
      templateFn,
      templateContent,
      templateParameters,
      uid,
    };
  },
);

function htmlTagObjectToString(tag: {
  tagName: string;
  attributes: Record<string, string>;
  voidTag: boolean;
  innerHTML?: string;
}) {
  const attributes = Object.keys(tag.attributes || {})
    .filter(
      (attributeName) =>
        tag.attributes[attributeName] === '' || tag.attributes[attributeName],
    )
    .map((attributeName) => {
      if (tag.attributes[attributeName] === 'true') {
        return attributeName;
      }
      return `${attributeName}="${tag.attributes[attributeName]}"`;
    });
  const res = `<${[tag.tagName].concat(attributes).join(' ')}${tag.voidTag && !tag.innerHTML ? '/' : ''}>${tag.innerHTML || ''}${tag.voidTag && !tag.innerHTML ? '' : `</${tag.tagName}>`}`;
  return res;
}

const HtmlRspackPlugin = HtmlRspackPluginImpl as typeof HtmlRspackPluginImpl & {
  /**
   * @deprecated Use `getCompilationHooks` instead.
   */
  getHooks: (compilation: Compilation) => HtmlRspackPluginHooks;
  getCompilationHooks: (compilation: Compilation) => HtmlRspackPluginHooks;
  createHtmlTagObject: (
    tagName: string,
    attributes?: Record<string, string | boolean>,
    innerHTML?: string,
  ) => JsHtmlPluginTag;
  version: number;
};

const voidTags = [
  'area',
  'base',
  'br',
  'col',
  'embed',
  'hr',
  'img',
  'input',
  'keygen',
  'link',
  'meta',
  'param',
  'source',
  'track',
  'wbr',
];

HtmlRspackPlugin.createHtmlTagObject = (
  tagName: string,
  attributes?: Record<string, string | boolean>,
  innerHTML?: string,
): JsHtmlPluginTag => {
  return {
    tagName,
    voidTag: voidTags.includes(tagName),
    attributes: attributes || {},
    innerHTML,
  };
};

HtmlRspackPlugin.getHooks = HtmlRspackPlugin.getCompilationHooks =
  getPluginHooks;
HtmlRspackPlugin.version = 5;

export { HtmlRspackPlugin };
