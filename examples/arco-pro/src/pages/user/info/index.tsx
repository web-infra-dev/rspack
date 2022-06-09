import useLocale from '@/utils/useLocale'
import { Grid, Link, Result, Skeleton, Typography } from '@arco-design/web-react'
import { Card } from '@arco-design/web-react'
import axios from 'axios'
import React, { useEffect, useState } from 'react'
import { useSelector } from 'react-redux'
import UserInfoHeader from './header'
import LatestNews from './latest-news'
import locale from './locale'
import './mock'
import MyProject from './my-projects'
import MyTeam from './my-team'
import styles from './style/index.module.less'

const { Title } = Typography
const { Row, Col } = Grid
function UserInfo() {
  const t = useLocale(locale)
  const userInfo = useSelector((state: any) => state.userInfo)
  const loading = useSelector((state: any) => state.userLoading)

  const [noticeLoading, setNoticeLoading] = useState(false)

  const getNotice = async () => {
    setNoticeLoading(true)
    await axios.get('/api/user/notice').finally(() => setNoticeLoading(false))
  }

  useEffect(() => {
    getNotice()
  }, [])

  return (
    <div>
      <UserInfoHeader userInfo={userInfo} loading={loading} />
      <Row gutter={16}>
        <Col span={16}>
          <Card className={styles.wrapper}>
            <div className={styles['card-title-wrapper']}>
              <Title heading={6} style={{ marginBottom: '20px' }}>
                {t['userInfo.title.project']}
              </Title>
              <Link>{t['userInfo.btn.more']}</Link>
            </div>
            <MyProject />
          </Card>
        </Col>
        <Col span={8}>
          <Card className={styles.wrapper}>
            <div className={styles['card-title-wrapper']}>
              <Title heading={6} style={{ marginBottom: '12px' }}>
                {t['userInfo.title.team']}
              </Title>
            </div>
            <MyTeam />
          </Card>
        </Col>
      </Row>
      <Row gutter={16}>
        <Col span={16}>
          <Card className={styles.wrapper}>
            <div className={styles['card-title-wrapper']}>
              <Title heading={6} style={{ marginBottom: '8px' }}>
                {t['userInfo.title.news']}
              </Title>
              <Link>{t['userInfo.btn.all']}</Link>
            </div>
            <LatestNews />
          </Card>
        </Col>
        <Col span={8}>
          <Card className={styles.wrapper}>
            <div className={styles['card-title-wrapper']}>
              <Title heading={6}>{t['userInfo.title.notice']}</Title>
            </div>
            {noticeLoading ? <Skeleton text={{ rows: 10 }} animation /> : (
              <Result
                status="404"
                subTitle={t['userInfo.notice.empty']}
                style={{ paddingTop: '60px', paddingBottom: '130px' }}
              />
            )}
          </Card>
        </Col>
      </Row>
    </div>
  )
}

export default UserInfo
