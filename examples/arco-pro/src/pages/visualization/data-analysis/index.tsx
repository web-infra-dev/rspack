import React, { useEffect, useMemo, useState } from 'react';
import { Card, Grid, Table, Space, Typography } from '@arco-design/web-react';
import useLocale from '@/utils/useLocale';
import axios from 'axios';
import locale from './locale';
import PublicOpinion from './public-opinion';
import MultiInterval from '@/components/Chart/multi-stack-interval';
import PeriodLine from '@/components/Chart/period-legend-line';
import './mock';

const { Row, Col } = Grid;

function DataAnalysis() {
  const t = useLocale(locale);
  const [loading, setLoading] = useState(false);
  const [tableLoading, setTableLoading] = useState(false);

  const [chartData, setChartData] = useState([]);
  const [tableData, setTableData] = useState([]);

  const getChartData = async () => {
    setLoading(true);
    const { data } = await axios
      .get('/api/data-analysis/content-publishing')
      .finally(() => setLoading(false));
    setChartData(data);
  };

  const getTableData = async () => {
    setTableLoading(true);
    const { data } = await axios
      .get('/api/data-analysis/author-list')
      .finally(() => setTableLoading(false));
    setTableData(data.list);
  };

  useEffect(() => {
    getChartData();
    getTableData();
  }, []);

  const columns = useMemo(() => {
    return [
      {
        title: t['dataAnalysis.authorTable.rank'],
        dataIndex: 'id',
      },
      {
        title: t['dataAnalysis.authorTable.author'],
        dataIndex: 'author',
      },
      {
        title: t['dataAnalysis.authorTable.content'],
        dataIndex: 'contentCount',
        sorter: (a, b) => a.contentCount - b.contentCount,
        render(x) {
          return Number(x).toLocaleString();
        },
      },
      {
        title: t['dataAnalysis.authorTable.click'],
        dataIndex: 'clickCount',
        sorter: (a, b) => a.clickCount - b.clickCount,
        render(x) {
          return Number(x).toLocaleString();
        },
      },
    ];
  }, [t]);

  return (
    <Space size={16} direction="vertical" style={{ width: '100%' }}>
      <Card>
        <Typography.Title heading={6}>
          {t['dataAnalysis.title.publicOpinion']}
        </Typography.Title>
        <PublicOpinion />
      </Card>
      <Row gutter={16}>
        <Col span={14}>
          <Card>
            <Typography.Title heading={6}>
              {t['dataAnalysis.title.publishingRate']}
            </Typography.Title>
            <MultiInterval data={chartData} loading={loading} />
          </Card>
        </Col>
        <Col span={10}>
          <Card>
            <Typography.Title heading={6}>
              {t['dataAnalysis.title.authorsList']}
            </Typography.Title>
            <div style={{ height: '370px' }}>
              <Table
                rowKey="id"
                loading={tableLoading}
                pagination={false}
                data={tableData}
                columns={columns}
              />
            </div>
          </Card>
        </Col>
      </Row>
      <Row>
        <Col span={24}>
          <Card>
            <Typography.Title heading={6}>
              {t['dataAnalysis.title.publishingTiming']}
            </Typography.Title>
            <PeriodLine data={chartData} loading={loading} />
          </Card>
        </Col>
      </Row>
    </Space>
  );
}
export default DataAnalysis;
