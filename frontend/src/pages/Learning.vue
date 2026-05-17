<template>
  <div class="learning-page">
    <section class="learn-hero neu-convex">
      <div>
        <span class="eyebrow">股票学习</span>
        <h2>把看盘时遇到的问题讲清楚</h2>
        <p>
          先理解方向、时段、成交量和流动性，再去看价格波动。这里是学习笔记，不是投资建议。
        </p>
      </div>
      <div class="session-clock">
        <span>美股常规交易</span>
        <strong>9:30 - 16:00 ET</strong>
        <small>北京时间夏令时 21:30 - 04:00，冬令时 22:30 - 05:00</small>
      </div>
    </section>

    <section class="topic-tabs" aria-label="学习分类">
      <button
        v-for="topic in topics"
        :key="topic.key"
        class="topic-tab"
        :class="{ 'topic-tab--active': activeTopic === topic.key }"
        type="button"
        @click="activeTopic = topic.key"
      >
        {{ topic.label }}
      </button>
    </section>

    <section v-if="activeTopic === 'basics'" class="content-grid">
      <NeuCard v-for="item in basics" :key="item.title" class="lesson-card">
        <span class="lesson-tag">{{ item.tag }}</span>
        <h3>{{ item.title }}</h3>
        <p>{{ item.body }}</p>
        <ul>
          <li v-for="point in item.points" :key="point">{{ point }}</li>
        </ul>
      </NeuCard>
    </section>

    <section v-if="activeTopic === 'sessions'" class="session-layout">
      <NeuCard class="timeline-card">
        <div class="section-head">
          <div>
            <span class="lesson-tag">交易时段</span>
            <h3>美股一天不是只有一个市场</h3>
          </div>
        </div>
        <div class="time-table">
          <div class="time-row time-row--head">
            <span>阶段</span>
            <span>美东时间</span>
            <span>北京时间夏令时</span>
            <span>北京时间冬令时</span>
          </div>
          <div v-for="row in sessionRows" :key="row.name" class="time-row">
            <strong>{{ row.name }}</strong>
            <span>{{ row.eastern }}</span>
            <span>{{ row.chinaDst }}</span>
            <span>{{ row.chinaStd }}</span>
          </div>
        </div>
        <p class="session-note">
          注：扩展时段会因交易所、证券和券商规则不同而变化；这里按常见美股扩展交易窗口理解。
        </p>
      </NeuCard>

      <NeuCard class="lesson-card">
        <span class="lesson-tag">为什么是 9:30</span>
        <h3>这是交易所制度安排，不是自然规律</h3>
        <p>
          9:30 ET 开始常规交易，4:00 ET 收盘，是美国主要交易所长期采用的核心交易时段。这个时段把做市商、机构、散户、基金调仓、指数交易和新闻消化集中到同一片流动性里。
        </p>
        <ul>
          <li>开盘前会先累积隔夜新闻、财报、宏观数据和订单。</li>
          <li>9:30 的开盘机制帮助市场形成第一笔更有代表性的价格。</li>
          <li>4:00 收盘附近也有大量收盘价相关订单和基金再平衡需求。</li>
        </ul>
      </NeuCard>
    </section>

    <section v-if="activeTopic === 'liquidity'" class="content-grid">
      <NeuCard v-for="item in liquidityLessons" :key="item.title" class="lesson-card">
        <span class="lesson-tag">{{ item.tag }}</span>
        <h3>{{ item.title }}</h3>
        <p>{{ item.body }}</p>
        <ul>
          <li v-for="point in item.points" :key="point">{{ point }}</li>
        </ul>
      </NeuCard>
    </section>

    <section v-if="activeTopic === 'checklist'" class="checklist-grid">
      <NeuCard class="check-card">
        <span class="lesson-tag">盘前先问</span>
        <h3>这次波动来自哪里</h3>
        <ol>
          <li v-for="item in preMarketChecks" :key="item">{{ item }}</li>
        </ol>
      </NeuCard>
      <NeuCard class="check-card">
        <span class="lesson-tag">下单前先看</span>
        <h3>价格有没有成交条件</h3>
        <ol>
          <li v-for="item in orderChecks" :key="item">{{ item }}</li>
        </ol>
      </NeuCard>
      <NeuCard class="source-card">
        <span class="lesson-tag">资料来源</span>
        <h3>官方入口</h3>
        <a v-for="source in sources" :key="source.href" :href="source.href" target="_blank" rel="noreferrer">
          {{ source.label }}
        </a>
      </NeuCard>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import NeuCard from '../components/NeuCard.vue'

type TopicKey = 'basics' | 'sessions' | 'liquidity' | 'checklist'

interface Lesson {
  tag: string
  title: string
  body: string
  points: string[]
}

const activeTopic = ref<TopicKey>('basics')

const topics: Array<{ key: TopicKey; label: string }> = [
  { key: 'basics', label: '基础概念' },
  { key: 'sessions', label: '交易时间' },
  { key: 'liquidity', label: '成交量' },
  { key: 'checklist', label: '看盘清单' },
]

const basics: Lesson[] = [
  {
    tag: '方向',
    title: '什么是做多',
    body: '做多就是买入并持有资产，预期价格未来上涨。你赚的是卖出价高于买入价的差额。',
    points: [
      '最大亏损通常不会超过投入本金，极端情况是股票跌到接近 0。',
      '上涨空间理论上没有固定上限，但过程会有波动。',
      '看多不等于必须满仓，可以分批、设定风险边界。',
    ],
  },
  {
    tag: '方向',
    title: '什么是做空',
    body: '做空通常是借入股票卖出，预期未来价格下跌后再买回归还。',
    points: [
      '如果股价下跌，买回成本更低，差额可能成为收益。',
      '如果股价上涨，亏损可能持续扩大，理论风险高于普通买入。',
      '做空通常涉及保证金、借券成本和被强制平仓风险。',
    ],
  },
  {
    tag: '价格',
    title: '什么是买一卖一价差',
    body: '买一是市场愿意买入的最高价，卖一是市场愿意卖出的最低价，两者差距就是价差。',
    points: [
      '价差越窄，说明买卖双方更容易达成交易。',
      '价差越宽，市价单更容易滑点。',
      '盘前盘后价差常常更宽，所以限价单更重要。',
    ],
  },
]

const sessionRows = [
  {
    name: '盘前',
    eastern: '04:00 - 09:30',
    chinaDst: '16:00 - 21:30',
    chinaStd: '17:00 - 22:30',
  },
  {
    name: '常规交易',
    eastern: '09:30 - 16:00',
    chinaDst: '21:30 - 04:00',
    chinaStd: '22:30 - 05:00',
  },
  {
    name: '盘后',
    eastern: '16:00 - 20:00',
    chinaDst: '04:00 - 08:00',
    chinaStd: '05:00 - 09:00',
  },
]

const liquidityLessons: Lesson[] = [
  {
    tag: '流动性',
    title: '为什么盘前盘后成交量少',
    body: '盘前盘后参与者更少，做市商和 ECN 参与也可能不是强制的，同一价格附近排队的人少，所以成交量自然下降。',
    points: [
      '买卖盘深度变薄，同样一笔订单更容易推动价格。',
      '价差通常变宽，看起来有价格，但未必能按理想价格成交。',
      '很多券商会限制盘前盘后的订单类型和可交易品种。',
    ],
  },
  {
    tag: '开盘',
    title: '为什么开盘附近量会突然变大',
    body: '隔夜新闻、财报、宏观数据、分析师调级和机构订单会在开盘前累积，9:30 后集中进入常规交易流动性池。',
    points: [
      '开盘价是大量买卖意愿重新撮合后的结果。',
      '开盘前几分钟波动大，不一定代表全天方向。',
      '开盘成交量大时，要同时看价格是否延续、回落还是横盘。',
    ],
  },
  {
    tag: '收盘',
    title: '为什么收盘附近也常常放量',
    body: '很多基金和指数产品需要用收盘价估值或调仓，交易者也会在收盘前处理隔夜风险。',
    points: [
      '4:00 ET 收盘价对净值、指数和业绩统计很重要。',
      '尾盘放量不一定是单一方向，可能是再平衡或对冲。',
      '观察尾盘时，要把价格变化和成交量一起看。',
    ],
  },
]

const preMarketChecks = [
  '昨晚有没有财报、指引、回购、并购或监管消息。',
  '宏观数据和利率预期有没有改变市场风险偏好。',
  '盘前价格有没有成交量支撑，还是只有很薄的报价。',
  '同板块股票是否同步变化，还是只有单只股票异动。',
]

const orderChecks = [
  '当前价差是否很宽，是否必须用限价单。',
  '成交量是否足够承接自己的订单规模。',
  '这笔交易是看方向、避险，还是只是被短线波动带着跑。',
  '如果价格反向走，自己准备在哪里止损或减仓。',
]

const sources = [
  {
    label: 'Investor.gov: Long and Short',
    href: 'https://www.investor.gov/introduction-investing/investing-basics/how-stock-markets-work/stock-purchases-and-sales-long-and',
  },
  {
    label: 'Investor.gov: After-hours Trading',
    href: 'https://www.investor.gov/introduction-investing/investing-basics/glossary/after-hours-trading',
  },
  {
    label: 'SEC Investor Bulletin: Extended-Hours Trading',
    href: 'https://www.investor.gov/introduction-investing/general-resources/news-alerts/alerts-bulletins/investor-bulletins-42',
  },
  {
    label: 'NYSE: Trading Hours and Calendars',
    href: 'https://www.nyse.com/markets/hours-calendars',
  },
  {
    label: 'Nasdaq Trader: Opening and Closing Crosses',
    href: 'https://www.nasdaqtrader.com/trader.aspx?id=openclose',
  },
]
</script>

<style scoped>
.learning-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.learn-hero {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(280px, 0.6fr);
  gap: 20px;
  align-items: center;
  padding: 22px;
}

.eyebrow,
.lesson-tag {
  color: var(--neu-primary);
  font-size: 13px;
  font-weight: 800;
}

.learn-hero h2,
.lesson-card h3,
.timeline-card h3,
.check-card h3,
.source-card h3 {
  margin: 8px 0 0;
  font-size: 24px;
  font-weight: 800;
}

.learn-hero p,
.lesson-card p {
  margin: 10px 0 0;
  max-width: 760px;
  color: var(--neu-text-dim);
  line-height: 1.75;
}

.session-clock {
  display: flex;
  min-height: 136px;
  flex-direction: column;
  justify-content: center;
  border-radius: var(--neu-radius);
  padding: 18px;
  background: var(--neu-bg);
  box-shadow:
    inset 4px 4px 8px var(--neu-shadow-dark),
    inset -4px -4px 8px var(--neu-shadow-light);
}

.session-clock span,
.session-clock small {
  color: var(--neu-text-dim);
  font-size: 13px;
}

.session-clock strong {
  margin: 8px 0;
  color: var(--neu-text);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0;
}

.topic-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.topic-tab {
  border: none;
  border-radius: var(--neu-radius-sm);
  padding: 11px 16px;
  background: var(--neu-bg);
  color: var(--neu-text-dim);
  cursor: pointer;
  font-weight: 800;
  box-shadow:
    3px 3px 6px var(--neu-shadow-dark),
    -3px -3px 6px var(--neu-shadow-light);
}

.topic-tab--active {
  color: #fff;
  background: var(--neu-primary);
  box-shadow: none;
}

.content-grid,
.checklist-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 22px;
}

.lesson-card {
  min-height: 280px;
}

.lesson-card ul,
.check-card ol {
  display: flex;
  flex-direction: column;
  gap: 9px;
  margin: 16px 0 0;
  padding-left: 20px;
  color: var(--neu-text);
  line-height: 1.65;
}

.session-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.35fr) minmax(280px, 0.65fr);
  gap: 22px;
}

.section-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
}

.time-table {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 20px;
}

.session-note {
  margin: 14px 0 0;
  color: var(--neu-text-dim);
  font-size: 13px;
  line-height: 1.7;
}

.time-row {
  display: grid;
  grid-template-columns: 0.8fr repeat(3, 1fr);
  gap: 12px;
  align-items: center;
  border-radius: var(--neu-radius-sm);
  padding: 14px 16px;
  background: var(--neu-bg);
  box-shadow:
    inset 2px 2px 5px var(--neu-shadow-dark),
    inset -2px -2px 5px var(--neu-shadow-light);
}

.time-row span {
  min-width: 0;
  color: var(--neu-text-dim);
  font-size: 14px;
}

.time-row strong {
  color: var(--neu-primary);
}

.time-row--head {
  box-shadow: none;
  background: transparent;
  font-weight: 800;
}

.check-card,
.source-card {
  min-height: 240px;
}

.source-card {
  display: flex;
  flex-direction: column;
}

.source-card a {
  margin-top: 12px;
  color: var(--neu-primary);
  font-weight: 700;
  text-decoration: none;
}

.source-card a:hover {
  text-decoration: underline;
}

@media (max-width: 920px) {
  .learn-hero,
  .session-layout {
    grid-template-columns: 1fr;
  }

  .time-row {
    grid-template-columns: 1fr;
  }

  .time-row--head {
    display: none;
  }
}
</style>
