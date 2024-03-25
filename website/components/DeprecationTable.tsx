import semver from 'semver';

import { useLang } from 'rspress/runtime';

import { Table } from '@builtIns/Table';
import { useLocation } from '@hooks/useLocation';

function useDeprecatedVersion() {
  const { query } = useLocation();
  return query.get('deprecatedVersion');
}

function DeprecationTable() {
  const depVer = useDeprecatedVersion();
  const lang = useLang();
  const en = lang === 'en';

  const DEPRECATION_HEADER = [
    {
      name: en ? 'Stage' : '阶段',
      key: 'stage',
    },
    {
      name: en ? 'Version' : '版本',
      key: 'version',
    },
    {
      name: en ? 'Description' : '描述',
      key: 'description',
    },
  ];

  const generateBody = () => {
    const next = depVer ? semver.inc(depVer, 'minor')! : '';
    const nextnext = next ? semver.inc(next, 'minor')! : '';

    if (en) {
      return [
        {
          stage: (
            <>
              <b>Deprecated</b>
              <p>
                <i>default value ~unchanged~</i>
              </p>
            </>
          ),
          version: depVer ? depVer : 'Current version',
          description:
            "It's open for migration. The default behavior remains **unchanged**.",
        },
        {
          stage: (
            <>
              <b>Deprecated</b>
              <p>
                <i>default value ~applied~</i>
              </p>
            </>
          ),
          version: next ? next : 'Next minor',
          description: (
            <>
              The default behavior is <b>changed</b> to the latest one. Turning
              off this new behavior is still an option, check out
              <a
                href="/config/experiments#experimentsrspackfuture"
                className="ml-2 mr-2"
                style={{ borderBottom: '1px dashed var(--rp-c-brand)' }}
              >
                experiments.rspackFuture
              </a>
              for the way to do it , but you should migrate to the new behavior
              as soon as possible.
            </>
          ),
        },
        {
          stage: <b>Removed</b>,
          version: nextnext ? nextnext : 'Minor/Major after next minor',
          description:
            'The migration should be completed. The old behavior and its corresponding option is **removed**. At the time, please refer to the migration guide or release note for the new behavior.',
        },
      ];
    }

    return [
      {
        stage: (
          <>
            <b>已废弃</b>
            <p>
              <i>保持默认行为</i>
            </p>
          </>
        ),
        version: depVer ? depVer : '当前版本',
        description: '可以进行迁移，默认行为**未发生改变**。',
      },
      {
        stage: (
          <>
            <b>已废弃</b>
            <p>
              <i>默认行为已更新</i>
            </p>
          </>
        ),
        version: next ? next : '下一个 minor 版本',
        description: (
          <>
            默认行为 <b>已更新</b>{' '}
            至新行为。但你仍旧可以关闭这个行为，具体请参考：
            <a
              href={`/${lang}/config/experiments#experimentsrspackfuture`}
              className="ml-2 mr-2"
              style={{ borderBottom: '1px dashed var(--rp-c-brand)' }}
            >
              experiments.rspackFuture
            </a>
            。请尽快迁移至新行为，以免功能不可用。
          </>
        ),
      },
      {
        stage: <b>已移除</b>,
        version: nextnext ? nextnext : '下下个 minor 或 major 版本',
        description:
          '需要完成迁移后才能使用该版本。旧的行为和对应的选项**已移除**。请参考迁移指南或者发布日志以迁移至新的行为。',
      },
    ];
  };

  return <Table header={DEPRECATION_HEADER} body={generateBody()} />;
}

export { DeprecationTable };
