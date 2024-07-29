import { Suspense, useEffect, useState } from 'react';
import './CompatibleCard.scss';
import axios from 'axios';
import * as i18n from './i18n';

interface CardMeta {
  name: string;
  url: string;
  rspackMinVersion: string;
  remark?: string;
}

const CompatibleCardItem = ({
  name,
  url,
  remark,
  rspackMinVersion,
}: CardMeta) => {
  return (
    <div className="component-card">
      <div className="component-card-title-line">
        <a
          className="component-card-link"
          target="_blank"
          rel="noreferrer"
          href={url}
        >
          {name}
        </a>
        <div className="component-card-space" />
        <div className="component-card-status">{rspackMinVersion}</div>
      </div>
      {remark && <div>{remark}</div>}
    </div>
  );
};

interface RspackCompatItem {
  name: string;
  version: string;
  rspackVersion: string;
  path: string;
}

export const CompatibleCardList = () => {
  const [loading, setLoading] = useState(true);
  const [list, setList] = useState<RspackCompatItem[]>([]);

  useEffect(() => {
    const url =
      'https://raw.githubusercontent.com/web-infra-dev/rspack-compat/data/rspack-compat.json';
    setLoading(true);
    axios.get<RspackCompatItem[]>(url).then(res => {
      const data = res.data.slice();
      data.sort((a, b) => {
        if (a.name < b.name) {
          return -1;
        }
        return 1;
      });
      setList(data);
      setLoading(false);
    });
  }, []);

  if (loading) {
    return <div />;
  }

  const prefix = 'https://github.com/web-infra-dev/rspack-compat/tree/main';
  return list.map(item => (
    <CompatibleCardItem
      key={item.name}
      name={`${item.name}@${item.version}`}
      url={`${prefix}/${item.path}`}
      rspackMinVersion={item.rspackVersion}
    />
  ));
};
