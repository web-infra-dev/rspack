import React, { useEffect, useState, useMemo } from 'react';
import {
  Statistic,
  Typography,
  Spin,
  Grid,
  Card,
  Skeleton,
} from '@arco-design/web-react';
import cs from 'classnames';
import { Chart, Line, Interval, Tooltip, Interaction } from 'bizcharts';
import axios from 'axios';
import useLocale from '@/utils/useLocale';
import locale from './locale';

import { IconArrowRise, IconArrowFall } from '@arco-design/web-react/icon';
import styles from './style/card-block.module.less';

const { Row, Col } = Grid;
const { Title, Text } = Typography;
const basicChartProps = {
  pure: true,
  autoFit: true,
  height: 80,
  padding: [0, 10, 0, 10],
};

export interface CardProps {
  key: string;
  title?: string;
  chartData?: any[];
  chartType: string;
  count?: number;
  increment?: boolean;
  diff?: number;
  loading?: boolean;
}

function CustomTooltip(props: { items: any[] }) {
  const { items } = props;
  return (
    <div className={styles.tooltip}>
      {items.map((item, index) => (
        <div key={index}>
          <Text bold>{Number(item.data.y).toLocaleString()}</Text>
        </div>
      ))}
    </div>
  );
}
function SimpleLine(props: { chartData: any[] }) {
  const { chartData } = props;
  return (
    <Chart data={chartData} {...basicChartProps}>
      <Line
        position="x*y"
        shape={['name', ['smooth', 'dash']]}
        color={['name', ['#165DFF', 'rgba(106,161,255,0.3)']]}
      />
      <Tooltip shared={false} showCrosshairs={true}>
        {(_, items) => <CustomTooltip items={items} />}
      </Tooltip>
    </Chart>
  );
}

function SimpleInterval(props: { chartData: any[] }) {
  const { chartData } = props;
  return (
    <Chart data={chartData} {...basicChartProps}>
      <Interval
        position="x*y"
        color={[
          'x',
          (xVal) => {
            if (Number(xVal) % 2 === 0) {
              return '#86DF6C';
            }
            return '#468DFF';
          },
        ]}
      />
      <Tooltip shared={false}>
        {(_, items) => <CustomTooltip items={items} />}
      </Tooltip>
      <Interaction type="active-region" />
    </Chart>
  );
}

function CardBlock(props: CardProps) {
  const { chartType, title, count, increment, diff, chartData, loading } =
    props;

  return (
    <Card className={styles.card}>
      <div className={styles.statistic}>
        <Statistic
          title={
            <Title heading={6} className={styles.title}>
              {title}
            </Title>
          }
          loading={loading}
          value={count}
          extra={
            <div className={styles['compare-yesterday']}>
              {loading ? (
                <Skeleton
                  text={{ rows: 1 }}
                  style={{ width: '100px' }}
                  animation
                />
              ) : (
                <span
                  className={cs(styles['diff'], {
                    [styles['diff-increment']]: increment,
                  })}
                >
                  {diff}
                  {increment ? <IconArrowRise /> : <IconArrowFall />}
                </span>
              )}
            </div>
          }
          groupSeparator
        />
      </div>
      <div className={styles.chart}>
        <Spin style={{ width: '100%' }} loading={loading}>
          {chartType === 'interval' && <SimpleInterval chartData={chartData} />}
          {chartType === 'line' && <SimpleLine chartData={chartData} />}
        </Spin>
      </div>
    </Card>
  );
}

const cardInfo = [
  {
    key: 'userRetentionTrend',
    type: 'line',
  },
  {
    key: 'userRetention',
    type: 'interval',
  },
  {
    key: 'contentConsumptionTrend',
    type: 'line',
  },
  {
    key: 'contentConsumption',
    type: 'interval',
  },
];
function CardList() {
  const t = useLocale(locale);
  const [loading, setLoading] = useState(false);
  const [data, setData] = useState(
    cardInfo.map((item) => ({
      ...item,
      chartType: item.type,
    }))
  );

  const getData = async () => {
    const requestList = cardInfo.map(async (info) => {
      const { data } = await axios
        .get(`/api/multi-dimension/card?type=${info.type}`)
        .catch(() => ({ data: {} }));
      return {
        ...data,
        key: info.key,
        chartType: info.type,
      };
    });

    setLoading(true);
    const result = await Promise.all(requestList).finally(() =>
      setLoading(false)
    );
    setData(result);
  };

  useEffect(() => {
    getData();
  }, []);

  const formatData = useMemo(() => {
    return data.map((item) => ({
      ...item,
      title: t[`multiDAnalysis.cardList.${item.key}`],
    }));
  }, [t, data]);

  return (
    <Row gutter={16}>
      {formatData.map((item, index) => (
        <Col span={6} key={index}>
          <CardBlock {...item} loading={loading} />
        </Col>
      ))}
    </Row>
  );
}

export default CardList;
