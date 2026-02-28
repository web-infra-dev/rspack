import { useLang } from '@rspress/core/runtime';
import { Link } from '@rspress/core/theme';
import styles from './ApiMeta.module.scss';

/**
 * The Stability Index is learned from https://nodejs.org/api/documentation.html#stability-index
 */
export enum Stability {
  Deprecated = 'Deprecated', // The feature may emit warnings. Backward compatibility is not guaranteed.
  Removed = 'Removed',
  Experimental = 'Experimental', // The feature is not subject to semantic versioning rules
}

export interface ApiMetaProps {
  addedVersion?: string;
  deprecatedVersion?: string;
  removedVersion?: string;
  stability?: Stability;
  inline?: boolean;
  specific?: string[];
}

export function ApiMeta(props: ApiMetaProps) {
  const lang = useLang();
  const href = `/${lang}/misc/planning/future`;
  const tagStyle = props.inline ? styles.tagInline : styles.tag;
  const wrapperStyle = props.inline ? styles.wrapperInline : styles.wrapper;

  const getGitTagHref = (version: string) =>
    `https://github.com/web-infra-dev/rspack/releases/tag/v${version.replace('v', '')}`;

  return (
    <div className={`${wrapperStyle} rp-not-doc`}>
      {props.addedVersion && (
        <span className={`${tagStyle} ${styles.added}`}>
          <Link href={getGitTagHref(props.addedVersion)}>
            Added in v{props.addedVersion}
          </Link>
        </span>
      )}
      {props.deprecatedVersion && (
        <span className={`${tagStyle} ${styles.deprecated}`}>
          <Link href={href}>Deprecated in v{props.deprecatedVersion}</Link>
        </span>
      )}
      {props.removedVersion && (
        <span className={`${tagStyle} ${styles.removed}`}>
          <Link href={href}>Removed in v{props.removedVersion}</Link>
        </span>
      )}
      {props.stability && (
        <div
          className={`${tagStyle} ${
            props.stability ? styles[props.stability.toLowerCase()] : ''
          }`}
        >
          Stability: {props.stability}
        </div>
      )}
      {props.specific && props.specific.length > 0 && (
        <span className={`${tagStyle} ${styles.specific}`}>
          {props.specific.join('/')}&nbsp;
          {props.specific.length > 1 ? 'specific' : 'only'}
        </span>
      )}
    </div>
  );
}
