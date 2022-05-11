import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { Link, Card, Skeleton, Tag, Typography } from '@arco-design/web-react';
import useLocale from '@/utils/useLocale';
import locale from './locale';
import styles from './style/announcement.module.less';

function Announcement() {
  const [data, setData] = useState([]);
  const [loading, setLoading] = useState(true);

  const t = useLocale(locale);

  const fetchData = () => {
    setLoading(true);
    axios
      .get('/api/workplace/announcement')
      .then((res) => {
        setData(res.data);
      })
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    fetchData();
  }, []);

  function getTagColor(type) {
    switch (type) {
      case 'activity':
        return 'orangered';
      case 'info':
        return 'cyan';
      case 'notice':
        return 'arcoblue';
      default:
        return 'arcoblue';
    }
  }

  return (
    <Card>
      <div style={{ display: 'flex', justifyContent: 'space-between' }}>
        <Typography.Title heading={6}>
          {t['workplace.announcement']}
        </Typography.Title>
        <Link>{t['workplace.seeMore']}</Link>
      </div>
      <Skeleton loading={loading} text={{ rows: 5, width: '100%' }} animation>
        <div>
          {data.map((d) => (
            <div key={d.key} className={styles.item}>
              <Tag color={getTagColor(d.type)} size="small">
                {t[`workplace.${d.type}`]}
              </Tag>
              <span className={styles.link}>{d.content}</span>
            </div>
          ))}
        </div>
      </Skeleton>
    </Card>
  );
}

export default Announcement;
