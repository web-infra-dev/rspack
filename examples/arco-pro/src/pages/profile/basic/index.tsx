import React, { useEffect, useState } from 'react';
import {
  Card,
  Steps,
  Typography,
  Grid,
  Space,
  Button,
  Table,
  Badge,
} from '@arco-design/web-react';
import axios from 'axios';
import useLocale from '@/utils/useLocale';
import locale from './locale';
import ProfileItem from './item';
import styles from './style/index.module.less';
import './mock';

function BasicProfile() {
  const t = useLocale(locale);
  const [loading, setLoading] = useState(false);
  const [data, setData] = useState({ status: 1 });
  const [preLoading, setPreLoading] = useState(false);
  const [preData, setPreData] = useState({});
  const [tableLoading, setTableLoading] = useState(false);
  const [tableData, setTableData] = useState([]);

  function fetchData() {
    setLoading(true);
    axios
      .get('/api/basicProfile')
      .then((res) => {
        setData(res.data || {});
      })
      .finally(() => {
        setLoading(false);
      });
  }

  function fetchPreData() {
    setPreLoading(true);
    axios
      .get('/api/basicProfile')
      .then((res) => {
        setPreData(res.data || {});
      })
      .finally(() => {
        setPreLoading(false);
      });
  }

  function fetchTableData() {
    setTableLoading(true);
    axios
      .get('/api/adjustment')
      .then((res) => {
        setTableData(res.data);
      })
      .finally(() => {
        setTableLoading(false);
      });
  }
  useEffect(() => {
    fetchData();
    fetchPreData();
    fetchTableData();
  }, []);

  return (
    <div className={styles.container}>
      <Card>
        <Grid.Row justify="space-between" align="center">
          <Grid.Col span={16}>
            <Typography.Title heading={6}>
              {t['basicProfile.title.form']}
            </Typography.Title>
          </Grid.Col>
          <Grid.Col span={8} style={{ textAlign: 'right' }}>
            <Space>
              <Button>{t['basicProfile.cancel']}</Button>
              <Button type="primary">{t['basicProfile.goBack']}</Button>
            </Space>
          </Grid.Col>
        </Grid.Row>

        <Steps current={data.status} lineless className={styles.steps}>
          <Steps.Step title={t['basicProfile.steps.commit']} />
          <Steps.Step title={t['basicProfile.steps.approval']} />
          <Steps.Step title={t['basicProfile.steps.finish']} />
        </Steps>
      </Card>

      <ProfileItem
        title={t['basicProfile.title.currentParams']}
        data={data}
        type="current"
        loading={loading}
      />
      <ProfileItem
        title={t['basicProfile.title.originParams']}
        data={preData}
        type="origin"
        loading={preLoading}
      />
      <Card>
        <Typography.Title heading={6}>
          {t['basicProfile.adjustment.record']}
        </Typography.Title>
        <Table
          loading={tableLoading}
          data={tableData}
          columns={[
            {
              dataIndex: 'contentId',
              title: t['basicProfile.adjustment.contentId'],
            },
            {
              dataIndex: 'content',
              title: t['basicProfile.adjustment.content'],
            },
            {
              dataIndex: 'status',
              title: t['basicProfile.adjustment.status'],
              render: (status) => {
                if (status) {
                  return (
                    <Badge
                      status="success"
                      text={t['basicProfile.adjustment.success']}
                    />
                  );
                }

                return (
                  <Badge
                    status="processing"
                    text={t['basicProfile.adjustment.waiting']}
                  />
                );
              },
            },
            {
              dataIndex: 'updatedTime',
              title: t['basicProfile.adjustment.updatedTime'],
            },
            {
              title: t['basicProfile.adjustment.operation'],
              headerCellStyle: { paddingLeft: '15px' },
              render() {
                return (
                  <Button type="text">
                    {t['basicProfile.adjustment.operation.view']}
                  </Button>
                );
              },
            },
          ]}
        />
      </Card>
    </div>
  );
}

export default BasicProfile;
