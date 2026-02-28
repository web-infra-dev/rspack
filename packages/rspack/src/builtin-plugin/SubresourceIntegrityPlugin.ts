import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join, relative, sep } from 'node:path';
import {
  BuiltinPluginName,
  type RawIntegrityData,
  type RawSubresourceIntegrityPluginOptions,
} from '@rspack/binding';
import type { AsyncSeriesWaterfallHook } from '@rspack/lite-tapable';
import type { Compilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import type { CrossOriginLoading } from '../config/types';
import { create } from './base';

const require = createRequire(import.meta.url);

const PLUGIN_NAME = 'SubresourceIntegrityPlugin';
const NATIVE_HTML_PLUGIN = 'HtmlRspackPlugin';

const HTTP_PROTOCOL_REGEX = /^https?:/;

type HtmlTagObject = {
  attributes: {
    [attributeName: string]: string | boolean | null | undefined;
  };
  tagName: string;
  innerHTML?: string;
  voidTag: boolean;
  meta: {
    plugin?: string;
    [metaAttributeName: string]: unknown;
  };
};

type BeforeAssetTagGenerationData = {
  assets: {
    publicPath: string;
    js: string[];
    css: string[];
    favicon?: string;
    manifest?: string;
    [extraAssetType: string]: unknown;
  };
  outputName: string;
  plugin: unknown;
};

type AlterAssetTagGroupsData = {
  headTags: HtmlTagObject[];
  bodyTags: HtmlTagObject[];
  outputName: string;
  publicPath: string;
  plugin: unknown;
};

type HtmlPluginHooks = {
  beforeAssetTagGeneration: AsyncSeriesWaterfallHook<BeforeAssetTagGenerationData>;
  alterAssetTagGroups: AsyncSeriesWaterfallHook<AlterAssetTagGroupsData>;
};

export type SubresourceIntegrityHashFunction = 'sha256' | 'sha384' | 'sha512';
export type SubresourceIntegrityPluginOptions = {
  hashFuncNames?: [
    SubresourceIntegrityHashFunction,
    ...SubresourceIntegrityHashFunction[],
  ];
  htmlPlugin?: string | false;
  enabled?: 'auto' | boolean;
};

export type NativeSubresourceIntegrityPluginOptions = Omit<
  RawSubresourceIntegrityPluginOptions,
  'htmlPlugin'
> & {
  htmlPlugin: string | false;
};

/**
 * Note: This is not a webpack public API, maybe removed in future.
 * @internal
 */
const NativeSubresourceIntegrityPlugin = create(
  BuiltinPluginName.SubresourceIntegrityPlugin,
  function (
    this: Compiler,
    options: NativeSubresourceIntegrityPluginOptions,
  ): RawSubresourceIntegrityPluginOptions {
    let htmlPlugin: RawSubresourceIntegrityPluginOptions['htmlPlugin'] =
      'Disabled';
    if (options.htmlPlugin === NATIVE_HTML_PLUGIN) {
      htmlPlugin = 'Native';
    } else if (typeof options.htmlPlugin === 'string') {
      htmlPlugin = 'JavaScript';
    }
    return {
      hashFuncNames: options.hashFuncNames,
      htmlPlugin,
      integrityCallback: options.integrityCallback,
    };
  },
);

export class SubresourceIntegrityPlugin extends NativeSubresourceIntegrityPlugin {
  private integrities: Map<string, string> = new Map();
  private options: SubresourceIntegrityPluginOptions;

  constructor(options: SubresourceIntegrityPluginOptions = {}) {
    if (typeof options !== 'object') {
      throw new Error('SubResourceIntegrity: argument must be an object');
    }

    const finalOptions = {
      hashFuncNames: options.hashFuncNames ?? ['sha384'],
      htmlPlugin: options.htmlPlugin ?? NATIVE_HTML_PLUGIN,
      enabled: options.enabled ?? 'auto',
    };
    super({
      ...finalOptions,
      integrityCallback: (data: RawIntegrityData) => {
        this.integrities = new Map(
          data.integerities.map((item) => [item.asset, item.integrity]),
        );
      },
    });

    this.options = finalOptions as SubresourceIntegrityPluginOptions;
  }

  private isEnabled(compiler: Compiler) {
    if (this.options.enabled === 'auto') {
      return compiler.options.mode !== 'development';
    }
    return this.options.enabled;
  }

  private getIntegrityChecksumForAsset(src: string): string | undefined {
    if (this.integrities.has(src)) {
      return this.integrities.get(src);
    }

    const normalizedSrc = normalizePath(src);
    const normalizedKey = Array.from(this.integrities.keys()).find(
      (assetKey) => normalizePath(assetKey) === normalizedSrc,
    );

    return normalizedKey ? this.integrities.get(normalizedKey) : undefined;
  }

  private handleHwpPluginArgs({ assets }: BeforeAssetTagGenerationData) {
    const publicPath = assets.publicPath;
    const jsIntegrity = [];
    for (const asset of assets.js) {
      jsIntegrity.push(
        this.getIntegrityChecksumForAsset(
          relative(publicPath, decodeURIComponent(asset)),
        ),
      );
    }

    const cssIntegrity = [];
    for (const asset of assets.css) {
      cssIntegrity.push(
        this.getIntegrityChecksumForAsset(
          relative(publicPath, decodeURIComponent(asset)),
        ),
      );
    }

    assets.jsIntegrity = jsIntegrity;
    assets.cssIntegrity = cssIntegrity;
  }

  private handleHwpBodyTags(
    { headTags, bodyTags, publicPath }: AlterAssetTagGroupsData,
    outputPath: string,
    crossOriginLoading: CrossOriginLoading | undefined,
  ) {
    for (const tag of headTags.concat(bodyTags)) {
      this.processTag(tag, publicPath, outputPath, crossOriginLoading);
    }
  }

  private processTag(
    tag: HtmlTagObject,
    publicPath: string,
    outputPath: string,
    crossOriginLoading: CrossOriginLoading | undefined,
  ): void {
    if (tag.attributes && 'integrity' in tag.attributes) {
      return;
    }

    const tagSrc = getTagSrc(tag);
    if (!tagSrc) {
      return;
    }

    // Check if the tagSrc is an absolute URL (http/https or protocol-relative)
    let isUrlSrc = false;
    try {
      const url = new URL(tagSrc);
      isUrlSrc = url.protocol === 'http:' || url.protocol === 'https:';
    } catch (_) {
      isUrlSrc = tagSrc.startsWith('//');
    }

    let src = '';
    if (isUrlSrc) {
      // For absolute URLs, we need to check if they're under our publicPath
      // If publicPath is just "/" or empty, it means local resources
      // External absolute URLs should be skipped
      const isLocalPublicPath =
        !publicPath || publicPath === '/' || publicPath === './';

      if (isLocalPublicPath) {
        // Local publicPath, skip all external URLs
        return;
      }

      const protocolRelativePublicPath = publicPath.replace(
        HTTP_PROTOCOL_REGEX,
        '',
      );
      const protocolRelativeTagSrc = tagSrc.replace(HTTP_PROTOCOL_REGEX, '');

      // If the tag src doesn't start with publicPath, it's an external resource
      // Skip SRI for external resources not served from our publicPath
      if (!protocolRelativeTagSrc.startsWith(protocolRelativePublicPath)) {
        return;
      }

      // Extract the asset path relative to publicPath
      const tagSrcWithScheme = `http:${protocolRelativeTagSrc}`;
      const publicPathWithScheme = protocolRelativePublicPath.startsWith('//')
        ? `http:${protocolRelativePublicPath}`
        : protocolRelativePublicPath;
      src = relative(
        publicPathWithScheme,
        decodeURIComponent(tagSrcWithScheme),
      );
    } else {
      src = relative(publicPath, decodeURIComponent(tagSrc));
    }

    tag.attributes.integrity =
      this.getIntegrityChecksumForAsset(src) ||
      computeIntegrity(
        this.options.hashFuncNames!,
        readFileSync(join(outputPath, src)),
      );
    tag.attributes.crossorigin = crossOriginLoading || 'anonymous';
  }

  apply(compiler: Compiler): void {
    if (!this.isEnabled(compiler)) {
      return;
    }

    super.apply(compiler);

    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.statsFactory.tap(PLUGIN_NAME, (statsFactory) => {
        statsFactory.hooks.extract
          .for('asset')
          .tap(PLUGIN_NAME, (object, asset) => {
            const contenthash = asset.info?.contenthash;
            if (contenthash) {
              const shaHashes = (
                Array.isArray(contenthash) ? contenthash : [contenthash]
              ).filter((hash: unknown) => String(hash).match(/^sha[0-9]+-/));
              if (shaHashes.length > 0) {
                (
                  object as unknown as {
                    integrity: string;
                  }
                ).integrity = shaHashes.join(' ');
              }
            }
          });
      });
    });

    if (
      typeof this.options.htmlPlugin === 'string' &&
      this.options.htmlPlugin !== NATIVE_HTML_PLUGIN
    ) {
      const self = this;
      function bindingHtmlHooks(htmlPlugin: any) {
        const getHooks: (compilation: Compilation) => HtmlPluginHooks =
          htmlPlugin.getCompilationHooks || htmlPlugin.getHooks;

        if (typeof getHooks === 'function') {
          compiler.hooks.thisCompilation.tap(PLUGIN_NAME, (compilation) => {
            if (
              typeof compiler.options.output.chunkLoading === 'string' &&
              ['require', 'async-node'].includes(
                compiler.options.output.chunkLoading,
              )
            ) {
              return;
            }
            const hwpHooks = getHooks(compilation);
            hwpHooks.beforeAssetTagGeneration.tapPromise(
              PLUGIN_NAME,
              (data) => {
                self.handleHwpPluginArgs(data);
                return Promise.resolve(data);
              },
            );

            hwpHooks.alterAssetTagGroups.tapPromise(
              {
                name: PLUGIN_NAME,
                stage: 10000,
              },
              (data) => {
                self.handleHwpBodyTags(
                  data,
                  compiler.outputPath,
                  compiler.options.output.crossOriginLoading,
                );
                return Promise.resolve(data);
              },
            );
          });
        }
      }

      try {
        const htmlPlugin = IS_BROWSER
          ? compiler.__internal_browser_require(this.options.htmlPlugin)
          : require(this.options.htmlPlugin);
        bindingHtmlHooks(htmlPlugin);
      } catch (e) {
        if (
          !isErrorWithCode(e as Error) ||
          (e as Error & { code: string }).code !== 'MODULE_NOT_FOUND'
        ) {
          throw e;
        }
      }
    }
  }
}

function isErrorWithCode<T extends Error>(obj: T): boolean {
  return (
    obj instanceof Error &&
    'code' in obj &&
    ['string', 'undefined'].includes(typeof obj.code)
  );
}

/**
 * Get the `src` or `href` attribute of a tag if it is a script
 * or link tag that needs SRI.
 */
function getTagSrc(tag: HtmlTagObject): string | undefined {
  if (!tag.attributes) {
    return undefined;
  }

  // Handle script tags with src attribute
  if (tag.tagName === 'script' && typeof tag.attributes.src === 'string') {
    return tag.attributes.src;
  }

  // Handle link tags that need SRI
  if (tag.tagName === 'link' && typeof tag.attributes.href === 'string') {
    const rel = tag.attributes.rel;
    if (typeof rel !== 'string') {
      return undefined;
    }

    // Only process link tags that load actual resources
    const needsSRI =
      rel === 'stylesheet' ||
      rel === 'modulepreload' ||
      (rel === 'preload' &&
        (tag.attributes.as === 'script' || tag.attributes.as === 'style'));

    return needsSRI ? tag.attributes.href : undefined;
  }

  return undefined;
}

function computeIntegrity(
  hashFuncNames: SubresourceIntegrityHashFunction[],
  source: string | Buffer,
): string {
  const { createHash } = require('node:crypto');
  const result = hashFuncNames
    .map(
      (hashFuncName) =>
        `${hashFuncName}-${createHash(hashFuncName)
          .update(
            typeof source === 'string' ? Buffer.from(source, 'utf-8') : source,
          )
          .digest('base64')}`,
    )
    .join(' ');
  return result;
}

function normalizePath(path: string): string {
  return path.replace(/\?.*$/, '').split(sep).join('/');
}
