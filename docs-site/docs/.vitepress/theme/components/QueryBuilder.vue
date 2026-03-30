<script setup lang="ts">
import { ref, computed, watch } from 'vue'

// ─── Types ───────────────────────────────────────────────────────────────────

type DataMode = 'historical' | 'streaming'
type AssetType = 'stock' | 'option' | 'index' | 'calendar' | 'rate'
type Language = 'rust' | 'python'

interface DataTypeOption {
  id: string
  label: string
  description: string
}

interface Params {
  symbol: string
  start_date: string
  end_date: string
  date: string
  interval: string
  expiration: string
  strike: string
  right: 'C' | 'P'
  time: string
  year: string
  rate_symbol: string
}

// ─── State ───────────────────────────────────────────────────────────────────

const step = ref(1)
const dataMode = ref<DataMode | null>(null)
const assetType = ref<AssetType | null>(null)
const dataType = ref<string | null>(null)
const language = ref<Language | null>(null)

const params = ref<Params>({
  symbol: 'AAPL',
  start_date: '20240101',
  end_date: '20240131',
  date: '20240101',
  interval: '60000',
  expiration: '20240621',
  strike: '450',
  right: 'C',
  time: '093000',
  year: '2024',
  rate_symbol: 'SOFR',
})

const copied = ref(false)

// ─── Step navigation ─────────────────────────────────────────────────────────

function selectDataMode(mode: DataMode) {
  dataMode.value = mode
  assetType.value = null
  dataType.value = null
  language.value = null
  step.value = 2
}

function selectAssetType(asset: AssetType) {
  assetType.value = asset
  dataType.value = null
  language.value = null
  step.value = 3
}

function selectDataType(dt: string) {
  dataType.value = dt
  language.value = null
  step.value = 4
}

function selectLanguage(lang: Language) {
  language.value = lang
  step.value = 6
}

function goToStep(n: number) {
  if (n < step.value) step.value = n
}

// ─── Data-type option definitions ────────────────────────────────────────────

const dataTypeOptions = computed((): DataTypeOption[] => {
  if (!dataMode.value || !assetType.value) return []

  if (dataMode.value === 'historical') {
    switch (assetType.value) {
      case 'stock':
        return [
          { id: 'eod', label: 'End-of-Day (EOD)', description: 'Daily OHLCV bars' },
          { id: 'ohlc', label: 'Intraday OHLC', description: '1min / 5min / 15min bars' },
          { id: 'trade', label: 'Trades', description: 'Every trade tick' },
          { id: 'quote', label: 'Quotes', description: 'NBBO quotes' },
          { id: 'trade_quote', label: 'Trade + Quote', description: 'Combined trade & quote stream' },
          { id: 'snapshot_ohlc', label: 'Snapshot OHLC', description: 'Latest OHLC bar' },
          { id: 'snapshot_trade', label: 'Snapshot Trade', description: 'Latest trade' },
          { id: 'snapshot_quote', label: 'Snapshot Quote', description: 'Latest quote' },
          { id: 'snapshot_market_value', label: 'Snapshot Market Value', description: 'Market cap & value' },
          { id: 'at_time_trade', label: 'At-Time Trade', description: 'Trade at a specific time of day' },
          { id: 'at_time_quote', label: 'At-Time Quote', description: 'Quote at a specific time of day' },
        ]
      case 'option':
        return [
          { id: 'list_expirations', label: 'List Expirations', description: 'All expiration dates for a root' },
          { id: 'list_strikes', label: 'List Strikes', description: 'All strikes for an expiration' },
          { id: 'eod', label: 'End-of-Day (EOD)', description: 'Daily OHLCV bars' },
          { id: 'ohlc', label: 'Intraday OHLC', description: 'Intraday OHLC bars' },
          { id: 'trade', label: 'Trades', description: 'Every trade tick' },
          { id: 'quote', label: 'Quotes', description: 'NBBO quotes' },
          { id: 'greeks_all', label: 'Greeks (All)', description: 'Full Greeks snapshot' },
          { id: 'greeks_iv', label: 'Implied Volatility', description: 'IV only' },
          { id: 'open_interest', label: 'Open Interest', description: 'Open interest over time' },
        ]
      case 'index':
        return [
          { id: 'eod', label: 'End-of-Day (EOD)', description: 'Daily OHLCV' },
          { id: 'ohlc', label: 'Intraday OHLC', description: 'Intraday bars' },
          { id: 'price', label: 'Price', description: 'Tick-level index price' },
        ]
      case 'calendar':
        return [
          { id: 'open_today', label: 'Open Today', description: 'Is the market open right now?' },
          { id: 'on_date', label: 'On Date', description: 'Market hours for a specific date' },
          { id: 'year', label: 'Full Year', description: 'All trading days in a year' },
        ]
      case 'rate':
        return [
          { id: 'eod', label: 'Interest Rate EOD', description: 'Daily risk-free rate' },
        ]
    }
  }

  if (dataMode.value === 'streaming') {
    switch (assetType.value) {
      case 'stock':
        return [
          { id: 'quote_stream', label: 'Quote Stream', description: 'Live NBBO quotes per symbol' },
          { id: 'trade_stream', label: 'Trade Stream', description: 'Live trades per symbol' },
          { id: 'firehose', label: 'Full Trade Firehose', description: 'All stock trades across the market' },
        ]
      case 'option':
        return [
          { id: 'option_quote_stream', label: 'Quote Stream', description: 'Live quotes per contract' },
          { id: 'option_trade_stream', label: 'Trade Stream', description: 'Live trades per contract' },
          { id: 'option_firehose', label: 'Full Trade Firehose', description: 'All option trades across the market' },
        ]
      default:
        return []
    }
  }

  return []
})

// ─── Parameter visibility ─────────────────────────────────────────────────────

const needsSymbol = computed(() => {
  if (assetType.value === 'calendar') return false
  if (assetType.value === 'rate') return false
  if (dataType.value === 'firehose' || dataType.value === 'option_firehose') return false
  return true
})

const needsRateSymbol = computed(() => assetType.value === 'rate')

const needsDateRange = computed(() => {
  const dt = dataType.value
  if (!dt) return false
  const dateRangeTypes = ['eod']
  if (assetType.value === 'index' && dt === 'ohlc') return true
  if (assetType.value === 'rate') return true
  if (assetType.value === 'stock' && dt === 'at_time_trade') return true
  if (assetType.value === 'stock' && dt === 'at_time_quote') return true
  return dateRangeTypes.includes(dt)
})

const needsSingleDate = computed(() => {
  const dt = dataType.value
  if (!dt) return false
  const singleDateTypes = ['ohlc', 'trade', 'quote', 'trade_quote', 'greeks_all', 'greeks_iv', 'open_interest']
  if (assetType.value === 'index' && dt === 'price') return true
  if (assetType.value === 'calendar' && dt === 'on_date') return true
  return singleDateTypes.includes(dt)
})

const needsInterval = computed(() => {
  const dt = dataType.value
  if (!dt) return false
  return ['ohlc', 'quote', 'greeks_all', 'greeks_iv'].includes(dt)
})

const needsOptionParams = computed(
  () => assetType.value === 'option' &&
    !['list_expirations', 'list_strikes'].includes(dataType.value ?? '')
)

const needsExpiration = computed(
  () => assetType.value === 'option' &&
    !['list_expirations'].includes(dataType.value ?? '') &&
    dataMode.value === 'historical'
)

const needsOptionContractForStream = computed(
  () => dataMode.value === 'streaming' &&
    (dataType.value === 'option_quote_stream' || dataType.value === 'option_trade_stream')
)

const needsAtTime = computed(
  () => dataType.value === 'at_time_trade' || dataType.value === 'at_time_quote'
)

const needsYear = computed(() => dataType.value === 'year')

const isSnapshotMulti = computed(() =>
  ['snapshot_ohlc', 'snapshot_trade', 'snapshot_quote', 'snapshot_market_value'].includes(dataType.value ?? '')
)

const intervalOptions = [
  { label: 'Tick (raw)', value: '0' },
  { label: '1 minute', value: '60000' },
  { label: '5 minutes', value: '300000' },
  { label: '15 minutes', value: '900000' },
  { label: '30 minutes', value: '1800000' },
  { label: '1 hour', value: '3600000' },
]

// ─── Code generation ──────────────────────────────────────────────────────────

function rustContract(sym: string): string {
  return `Contract::stock("${sym}")`
}

function pythonOptionParams(): string {
  const isCall = params.value.right === 'C'
  return `"${params.value.symbol}", "${params.value.expiration}", ${isCall}, ${params.value.strike}`
}

function rustOptionContract(): string {
  const isCall = params.value.right === 'C' ? 'true' : 'false'
  return `Contract::option("${params.value.symbol}", "${params.value.expiration}", ${isCall}, ${params.value.strike})`
}

const generatedCode = computed((): string => {
  if (!dataMode.value || !assetType.value || !dataType.value || !language.value) return ''

  const p = params.value
  const isRust = language.value === 'rust'
  const sym = p.symbol.toUpperCase()
  const rateSym = p.rate_symbol.toUpperCase()

  // ── Streaming ──────────────────────────────────────────────────────────────
  if (dataMode.value === 'streaming') {
    if (isRust) {
      return generateRustStreaming(sym)
    } else {
      return generatePythonStreaming(sym)
    }
  }

  // ── Historical ─────────────────────────────────────────────────────────────
  if (isRust) {
    return generateRustHistorical(sym, rateSym)
  } else {
    return generatePythonHistorical(sym, rateSym)
  }
})

function generateRustHistorical(sym: string, rateSym: string): string {
  const p = params.value
  const dt = dataType.value!
  const asset = assetType.value!

  let method = ''
  let callParams = ''
  let tickType = ''
  let printBody = ''

  if (asset === 'stock') {
    switch (dt) {
      case 'eod':
        method = 'stock_history_eod'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}"`
        tickType = 'EodTick'
        printBody = `"{}: open={} high={} low={} close={} vol={}", tick.date, tick.open_price(), tick.high_price(), tick.low_price(), tick.close_price(), tick.volume`
        break
      case 'ohlc':
        method = 'stock_history_ohlc'
        callParams = `"${sym}", "${p.date}", ${p.interval}`
        tickType = 'OhlcTick'
        printBody = `"{} ms={}: open={} high={} low={} close={} vol={}", tick.date, tick.ms_of_day, tick.open_price(), tick.high_price(), tick.low_price(), tick.close_price(), tick.volume`
        break
      case 'trade':
        method = 'stock_history_trade'
        callParams = `"${sym}", "${p.date}"`
        tickType = 'TradeTick'
        printBody = `"{} ms={}: price={} size={}", tick.date, tick.ms_of_day, tick.price, tick.size`
        break
      case 'quote':
        method = 'stock_history_quote'
        callParams = `"${sym}", "${p.date}", ${p.interval}`
        tickType = 'QuoteTick'
        printBody = `"{} ms={}: bid={} ask={} bid_sz={} ask_sz={}", tick.date, tick.ms_of_day, tick.bid, tick.ask, tick.bid_size, tick.ask_size`
        break
      case 'trade_quote':
        method = 'stock_history_trade_quote'
        callParams = `"${sym}", "${p.date}"`
        tickType = 'TradeQuoteTick'
        printBody = `"{} ms={}: price={} size={} bid={} ask={}", tick.date, tick.ms_of_day, tick.price, tick.size, tick.bid, tick.ask`
        break
      case 'snapshot_ohlc':
        method = 'stock_snapshot_ohlc'
        callParams = `&["${sym}"]`
        tickType = 'OhlcTick'
        printBody = `"{} ms={}: open={} close={}", tick.date, tick.ms_of_day, tick.open_price(), tick.close_price()`
        break
      case 'snapshot_trade':
        method = 'stock_snapshot_trade'
        callParams = `&["${sym}"]`
        tickType = 'TradeTick'
        printBody = `"price={} size={}", tick.price, tick.size`
        break
      case 'snapshot_quote':
        method = 'stock_snapshot_quote'
        callParams = `&["${sym}"]`
        tickType = 'QuoteTick'
        printBody = `"bid={} ask={}", tick.bid, tick.ask`
        break
      case 'snapshot_market_value':
        method = 'stock_snapshot_market_value'
        callParams = `&["${sym}"]`
        tickType = 'MarketValueTick'
        printBody = `"market_cap={}", tick.market_cap`
        break
      case 'at_time_trade':
        method = 'stock_at_time_trade'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}", "${p.time}"`
        tickType = 'TradeTick'
        printBody = `"{}: price={} size={}", tick.date, tick.price, tick.size`
        break
      case 'at_time_quote':
        method = 'stock_at_time_quote'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}", "${p.time}"`
        tickType = 'QuoteTick'
        printBody = `"{}: bid={} ask={}", tick.date, tick.bid, tick.ask`
        break
    }
  } else if (asset === 'option') {
    const isCall = p.right === 'C' ? 'true' : 'false'
    const baseOptionParams = `"${sym}", "${p.expiration}", ${isCall}, ${p.strike}`
    switch (dt) {
      case 'list_expirations':
        method = 'option_list_expirations'
        callParams = `"${sym}"`
        tickType = 'String'
        printBody = `"{}", exp`
        return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    let expirations = tdx.${method}(${callParams}).await?;
    for exp in &expirations {
        println!("${printBody}");
    }
    Ok(())
}`
          .replace('${method}', method)
          .replace('${callParams}', callParams)
          .replace('${printBody}', printBody)

      case 'list_strikes':
        method = 'option_list_strikes'
        callParams = `"${sym}", "${p.expiration}"`
        tickType = 'f64'
        printBody = `"{}", strike`
        return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    let strikes = tdx.${method}(${callParams}).await?;
    for strike in &strikes {
        println!("${printBody}");
    }
    Ok(())
}`
          .replace('${method}', method)
          .replace('${callParams}', callParams)
          .replace('${printBody}', printBody)

      case 'eod':
        method = 'option_history_eod'
        callParams = `${baseOptionParams}, "${p.start_date}", "${p.end_date}"`
        tickType = 'EodTick'
        printBody = `"{}: open={} high={} low={} close={} vol={}", tick.date, tick.open_price(), tick.high_price(), tick.low_price(), tick.close_price(), tick.volume`
        break
      case 'ohlc':
        method = 'option_history_ohlc'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        tickType = 'OhlcTick'
        printBody = `"{} ms={}: open={} close={}", tick.date, tick.ms_of_day, tick.open_price(), tick.close_price()`
        break
      case 'trade':
        method = 'option_history_trade'
        callParams = `${baseOptionParams}, "${p.date}"`
        tickType = 'TradeTick'
        printBody = `"{} ms={}: price={} size={}", tick.date, tick.ms_of_day, tick.price, tick.size`
        break
      case 'quote':
        method = 'option_history_quote'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        tickType = 'QuoteTick'
        printBody = `"{} ms={}: bid={} ask={}", tick.date, tick.ms_of_day, tick.bid, tick.ask`
        break
      case 'greeks_all':
        method = 'option_history_greeks_all'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        tickType = 'GreeksTick'
        printBody = `"{} ms={}: iv={} delta={} gamma={} theta={} vega={}", tick.date, tick.ms_of_day, tick.implied_volatility, tick.delta, tick.gamma, tick.theta, tick.vega`
        break
      case 'greeks_iv':
        method = 'option_history_greeks_implied_volatility'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        tickType = 'IvTick'
        printBody = `"{} ms={}: iv={} iv_err={}", tick.date, tick.ms_of_day, tick.implied_volatility, tick.iv_error`
        break
      case 'open_interest':
        method = 'option_history_open_interest'
        callParams = `${baseOptionParams}, "${p.date}"`
        tickType = 'OpenInterestTick'
        printBody = `"{} ms={}: oi={}", tick.date, tick.ms_of_day, tick.open_interest`
        break
    }
  } else if (asset === 'index') {
    switch (dt) {
      case 'eod':
        method = 'index_history_eod'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}"`
        tickType = 'EodTick'
        printBody = `"{}: open={} high={} low={} close={} vol={}", tick.date, tick.open_price(), tick.high_price(), tick.low_price(), tick.close_price(), tick.volume`
        break
      case 'ohlc':
        method = 'index_history_ohlc'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}", ${p.interval}`
        tickType = 'OhlcTick'
        printBody = `"{} ms={}: open={} close={}", tick.date, tick.ms_of_day, tick.open_price(), tick.close_price()`
        break
      case 'price':
        method = 'index_history_price'
        callParams = `"${sym}", "${p.date}", ${p.interval}`
        tickType = 'PriceTick'
        printBody = `"{} ms={}: price={}", tick.date, tick.ms_of_day, tick.price`
        break
    }
  } else if (asset === 'calendar') {
    switch (dt) {
      case 'open_today':
        return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    let day = tdx.calendar_open_today().await?;
    println!("date={} open={} hours={}-{}", day.date, day.is_open, day.open_time, day.close_time);
    Ok(())
}`
      case 'on_date':
        return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    let day = tdx.calendar_on_date("${p.date}").await?;
    println!("date={} open={} hours={}-{}", day.date, day.is_open, day.open_time, day.close_time);
    Ok(())
}`
      case 'year':
        return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    let days = tdx.calendar_year(${p.year}).await?;
    for day in &days {
        println!("date={} open={} hours={}-{}", day.date, day.is_open, day.open_time, day.close_time);
    }
    Ok(())
}`
    }
  } else if (asset === 'rate') {
    return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    let ticks = tdx.interest_rate_history_eod("${rateSym}", "${p.start_date}", "${p.end_date}").await?;
    for tick in &ticks {
        println!("{}: rate={}", tick.date, tick.rate);
    }
    Ok(())
}`
  }

  if (!method) return '// No template available for this combination.'

  return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    let ticks = tdx.${method}(${callParams}).await?;
    for tick in &ticks {
        println!("${printBody}");
    }
    Ok(())
}`
    .replace('${method}', method)
    .replace('${callParams}', callParams)
    .replace('${printBody}', printBody)
}

function generatePythonHistorical(sym: string, rateSym: string): string {
  const p = params.value
  const dt = dataType.value!
  const asset = assetType.value!

  let method = ''
  let callParams = ''
  let printBody = ''

  if (asset === 'stock') {
    switch (dt) {
      case 'eod':
        method = 'stock_history_eod'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}"`
        printBody = `{tick['date']}: open={tick['open']} high={tick['high']} low={tick['low']} close={tick['close']} vol={tick['volume']}`
        break
      case 'ohlc':
        method = 'stock_history_ohlc'
        callParams = `"${sym}", "${p.date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: open={tick['open']} close={tick['close']}`
        break
      case 'trade':
        method = 'stock_history_trade'
        callParams = `"${sym}", "${p.date}"`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: price={tick['price']} size={tick['size']}`
        break
      case 'quote':
        method = 'stock_history_quote'
        callParams = `"${sym}", "${p.date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: bid={tick['bid']} ask={tick['ask']}`
        break
      case 'trade_quote':
        method = 'stock_history_trade_quote'
        callParams = `"${sym}", "${p.date}"`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: price={tick['price']} bid={tick['bid']} ask={tick['ask']}`
        break
      case 'snapshot_ohlc':
        method = 'stock_snapshot_ohlc'
        callParams = `["${sym}"]`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: open={tick['open']} close={tick['close']}`
        break
      case 'snapshot_trade':
        method = 'stock_snapshot_trade'
        callParams = `["${sym}"]`
        printBody = `price={tick['price']} size={tick['size']}`
        break
      case 'snapshot_quote':
        method = 'stock_snapshot_quote'
        callParams = `["${sym}"]`
        printBody = `bid={tick['bid']} ask={tick['ask']}`
        break
      case 'snapshot_market_value':
        method = 'stock_snapshot_market_value'
        callParams = `["${sym}"]`
        printBody = `market_cap={tick['market_cap']}`
        break
      case 'at_time_trade':
        method = 'stock_at_time_trade'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}", "${p.time}"`
        printBody = `{tick['date']}: price={tick['price']} size={tick['size']}`
        break
      case 'at_time_quote':
        method = 'stock_at_time_quote'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}", "${p.time}"`
        printBody = `{tick['date']}: bid={tick['bid']} ask={tick['ask']}`
        break
    }
  } else if (asset === 'option') {
    const baseOptionParams = `"${sym}", "${p.expiration}", ${p.right === 'C' ? 'True' : 'False'}, ${p.strike}`
    switch (dt) {
      case 'list_expirations':
        return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

expirations = tdx.option_list_expirations("${sym}")
for exp in expirations:
    print(exp)`

      case 'list_strikes':
        return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

strikes = tdx.option_list_strikes("${sym}", "${p.expiration}")
for strike in strikes:
    print(strike)`

      case 'eod':
        method = 'option_history_eod'
        callParams = `${baseOptionParams}, "${p.start_date}", "${p.end_date}"`
        printBody = `{tick['date']}: open={tick['open']} close={tick['close']} vol={tick['volume']}`
        break
      case 'ohlc':
        method = 'option_history_ohlc'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: open={tick['open']} close={tick['close']}`
        break
      case 'trade':
        method = 'option_history_trade'
        callParams = `${baseOptionParams}, "${p.date}"`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: price={tick['price']} size={tick['size']}`
        break
      case 'quote':
        method = 'option_history_quote'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: bid={tick['bid']} ask={tick['ask']}`
        break
      case 'greeks_all':
        method = 'option_history_greeks_all'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: iv={tick['implied_volatility']} delta={tick['delta']} gamma={tick['gamma']}`
        break
      case 'greeks_iv':
        method = 'option_history_greeks_implied_volatility'
        callParams = `${baseOptionParams}, "${p.date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: iv={tick['implied_volatility']} err={tick['iv_error']}`
        break
      case 'open_interest':
        method = 'option_history_open_interest'
        callParams = `${baseOptionParams}, "${p.date}"`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: oi={tick['open_interest']}`
        break
    }
  } else if (asset === 'index') {
    switch (dt) {
      case 'eod':
        method = 'index_history_eod'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}"`
        printBody = `{tick['date']}: open={tick['open']} high={tick['high']} low={tick['low']} close={tick['close']}`
        break
      case 'ohlc':
        method = 'index_history_ohlc'
        callParams = `"${sym}", "${p.start_date}", "${p.end_date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: open={tick['open']} close={tick['close']}`
        break
      case 'price':
        method = 'index_history_price'
        callParams = `"${sym}", "${p.date}", ${p.interval}`
        printBody = `{tick['date']} ms={tick['ms_of_day']}: price={tick['price']}`
        break
    }
  } else if (asset === 'calendar') {
    switch (dt) {
      case 'open_today':
        return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

day = tdx.calendar_open_today()
print(f"{day['date']}: open={day['is_open']} hours={day['open_time']}-{day['close_time']}")`

      case 'on_date':
        return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

day = tdx.calendar_on_date("${p.date}")
print(f"{day['date']}: open={day['is_open']} hours={day['open_time']}-{day['close_time']}")`

      case 'year':
        return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

days = tdx.calendar_year(${p.year})
for day in days:
    print(f"{day['date']}: open={day['is_open']}")`
    }
  } else if (asset === 'rate') {
    return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

ticks = tdx.interest_rate_history_eod("${rateSym}", "${p.start_date}", "${p.end_date}")
for tick in ticks:
    print(f"{tick['date']}: rate={tick['rate']}")`
  }

  if (!method) return '# No template available for this combination.'

  return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

ticks = tdx.${method}(${callParams})
for tick in ticks:
    print(f"${printBody}")`
    .replace('${method}', method)
    .replace('${callParams}', callParams)
    .replace('${printBody}', printBody)
}

function generateRustStreaming(sym: string): string {
  const p = params.value
  const dt = dataType.value!

  switch (dt) {
    case 'quote_stream':
      return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};
use thetadatadx::fpss::{FpssEvent, FpssData};
use thetadatadx::fpss::protocol::Contract;

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    tdx.start_streaming(|event: &FpssEvent| {
        match event {
            FpssEvent::Data(FpssData::Quote { bid, ask, bid_size, ask_size, ms_of_day, date, .. }) => {
                println!("{} ms={}: bid={} ask={} bid_sz={} ask_sz={}", date, ms_of_day, bid, ask, bid_size, ask_size);
            }
            _ => {}
        }
    })?;

    tdx.subscribe_quotes(&Contract::stock("${sym}"))?;

    // Keep alive
    std::thread::park();
    Ok(())
}`

    case 'trade_stream':
      return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};
use thetadatadx::fpss::{FpssEvent, FpssData};
use thetadatadx::fpss::protocol::Contract;

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    tdx.start_streaming(|event: &FpssEvent| {
        match event {
            FpssEvent::Data(FpssData::Trade { price, size, ms_of_day, date, .. }) => {
                println!("{} ms={}: price={} size={}", date, ms_of_day, price, size);
            }
            _ => {}
        }
    })?;

    tdx.subscribe_trades(&Contract::stock("${sym}"))?;

    // Keep alive
    std::thread::park();
    Ok(())
}`

    case 'firehose':
      return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};
use thetadatadx::fpss::{FpssEvent, FpssData, SecType};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    tdx.start_streaming(|event: &FpssEvent| {
        match event {
            FpssEvent::Data(FpssData::Trade { price, size, ms_of_day, date, .. }) => {
                println!("{} ms={}: price={} size={}", date, ms_of_day, price, size);
            }
            _ => {}
        }
    })?;

    tdx.subscribe_full_trades(SecType::Stock)?;

    // Keep alive
    std::thread::park();
    Ok(())
}`

    case 'option_quote_stream': {
      const isCall = p.right === 'C' ? 'true' : 'false'
      return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};
use thetadatadx::fpss::{FpssEvent, FpssData};
use thetadatadx::fpss::protocol::Contract;

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    tdx.start_streaming(|event: &FpssEvent| {
        match event {
            FpssEvent::Data(FpssData::Quote { bid, ask, bid_size, ask_size, ms_of_day, date, .. }) => {
                println!("{} ms={}: bid={} ask={} bid_sz={} ask_sz={}", date, ms_of_day, bid, ask, bid_size, ask_size);
            }
            _ => {}
        }
    })?;

    tdx.subscribe_quotes(&Contract::option("${p.symbol.toUpperCase()}", "${p.expiration}", ${isCall}, ${p.strike}))?;

    // Keep alive
    std::thread::park();
    Ok(())
}`
    }

    case 'option_trade_stream': {
      const isCall = p.right === 'C' ? 'true' : 'false'
      return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};
use thetadatadx::fpss::{FpssEvent, FpssData};
use thetadatadx::fpss::protocol::Contract;

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    tdx.start_streaming(|event: &FpssEvent| {
        match event {
            FpssEvent::Data(FpssData::Trade { price, size, ms_of_day, date, .. }) => {
                println!("{} ms={}: price={} size={}", date, ms_of_day, price, size);
            }
            _ => {}
        }
    })?;

    tdx.subscribe_trades(&Contract::option("${p.symbol.toUpperCase()}", "${p.expiration}", ${isCall}, ${p.strike}))?;

    // Keep alive
    std::thread::park();
    Ok(())
}`
    }

    case 'option_firehose':
      return `use thetadatadx::{ThetaDataDx, Credentials, DirectConfig};
use thetadatadx::fpss::{FpssEvent, FpssData, SecType};

#[tokio::main]
async fn main() -> Result<(), thetadatadx::Error> {
    let creds = Credentials::from_file("creds.txt")?;
    let tdx = ThetaDataDx::connect(&creds, DirectConfig::production()).await?;

    tdx.start_streaming(|event: &FpssEvent| {
        match event {
            FpssEvent::Data(FpssData::Trade { price, size, ms_of_day, date, .. }) => {
                println!("{} ms={}: price={} size={}", date, ms_of_day, price, size);
            }
            _ => {}
        }
    })?;

    tdx.subscribe_full_trades(SecType::Option)?;

    // Keep alive
    std::thread::park();
    Ok(())
}`
  }

  return '// No template available for this combination.'
}

function generatePythonStreaming(sym: string): string {
  const p = params.value
  const dt = dataType.value!

  switch (dt) {
    case 'quote_stream':
      return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

tdx.start_streaming()
tdx.subscribe_quotes("${sym}")

while True:
    event = tdx.next_event(timeout_ms=5000)
    if event is None:
        continue
    if event["kind"] == "Quote":
        print(f"{event['date']} ms={event['ms_of_day']}: bid={event['bid']} ask={event['ask']}")

tdx.stop_streaming()`

    case 'trade_stream':
      return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

tdx.start_streaming()
tdx.subscribe_trades("${sym}")

while True:
    event = tdx.next_event(timeout_ms=5000)
    if event is None:
        continue
    if event["kind"] == "Trade":
        print(f"{event['date']} ms={event['ms_of_day']}: price={event['price']} size={event['size']}")

tdx.stop_streaming()`

    case 'firehose':
      return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

tdx.start_streaming()
tdx.subscribe_full_trades("STOCK")

while True:
    event = tdx.next_event(timeout_ms=5000)
    if event is None:
        continue
    if event["kind"] == "Trade":
        print(f"{event['date']} ms={event['ms_of_day']}: price={event['price']} size={event['size']}")

tdx.stop_streaming()`

    case 'option_quote_stream':
      return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

tdx.start_streaming()
tdx.subscribe_option_quotes("${p.symbol.toUpperCase()}", "${p.expiration}", ${p.right === 'C' ? 'True' : 'False'}, ${p.strike})

while True:
    event = tdx.next_event(timeout_ms=5000)
    if event is None:
        continue
    if event["kind"] == "Quote":
        print(f"{event['date']} ms={event['ms_of_day']}: bid={event['bid']} ask={event['ask']}")

tdx.stop_streaming()`

    case 'option_trade_stream':
      return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

tdx.start_streaming()
tdx.subscribe_option_trades("${p.symbol.toUpperCase()}", "${p.expiration}", ${p.right === 'C' ? 'True' : 'False'}, ${p.strike})

while True:
    event = tdx.next_event(timeout_ms=5000)
    if event is None:
        continue
    if event["kind"] == "Trade":
        print(f"{event['date']} ms={event['ms_of_day']}: price={event['price']} size={event['size']}")

tdx.stop_streaming()`

    case 'option_firehose':
      return `from thetadatadx import ThetaDataDx, Credentials, Config

creds = Credentials.from_file("creds.txt")
tdx = ThetaDataDx(creds, Config.production())

tdx.start_streaming()
tdx.subscribe_full_trades("OPTION")

while True:
    event = tdx.next_event(timeout_ms=5000)
    if event is None:
        continue
    if event["kind"] == "Trade":
        print(f"{event['date']} ms={event['ms_of_day']}: price={event['price']} size={event['size']}")

tdx.stop_streaming()`
  }

  return '# No template available for this combination.'
}

// ─── Code language label ──────────────────────────────────────────────────────

const codeLanguageLabel = computed(() =>
  language.value === 'rust' ? 'rust' : 'python'
)

// ─── Copy to clipboard ───────────────────────────────────────────────────────

async function copyCode() {
  if (!generatedCode.value) return
  try {
    await navigator.clipboard.writeText(generatedCode.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // fallback
    const el = document.createElement('textarea')
    el.value = generatedCode.value
    document.body.appendChild(el)
    el.select()
    document.execCommand('copy')
    document.body.removeChild(el)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  }
}

// ─── Reset ───────────────────────────────────────────────────────────────────

function reset() {
  dataMode.value = null
  assetType.value = null
  dataType.value = null
  language.value = null
  step.value = 1
}

// ─── Breadcrumb helpers ───────────────────────────────────────────────────────

const dataModeLabel = computed(() => {
  if (dataMode.value === 'historical') return 'Historical'
  if (dataMode.value === 'streaming') return 'Real-Time'
  return null
})

const assetLabel = computed(() => {
  const map: Record<AssetType, string> = {
    stock: 'Stock',
    option: 'Option',
    index: 'Index',
    calendar: 'Calendar',
    rate: 'Interest Rate',
  }
  return assetType.value ? map[assetType.value] : null
})

const dataTypeLabel = computed(() => {
  return dataTypeOptions.value.find(o => o.id === dataType.value)?.label ?? null
})

const languageLabel = computed(() => {
  if (language.value === 'rust') return 'Rust'
  if (language.value === 'python') return 'Python'
  return null
})
</script>

<template>
  <div class="qb-root">

    <!-- Breadcrumb / progress trail -->
    <div class="qb-trail" v-if="step > 1">
      <button class="qb-crumb qb-crumb--active" @click="reset">Start over</button>
      <span class="qb-crumb-sep">/</span>
      <button class="qb-crumb" :class="{ 'qb-crumb--done': step > 1 }" @click="goToStep(1)">
        {{ dataModeLabel ?? 'Mode' }}
      </button>
      <template v-if="step > 2">
        <span class="qb-crumb-sep">/</span>
        <button class="qb-crumb" :class="{ 'qb-crumb--done': step > 2 }" @click="goToStep(2)">
          {{ assetLabel ?? 'Asset' }}
        </button>
      </template>
      <template v-if="step > 3">
        <span class="qb-crumb-sep">/</span>
        <button class="qb-crumb" :class="{ 'qb-crumb--done': step > 3 }" @click="goToStep(3)">
          {{ dataTypeLabel ?? 'Data' }}
        </button>
      </template>
      <template v-if="step > 4">
        <span class="qb-crumb-sep">/</span>
        <button class="qb-crumb" :class="{ 'qb-crumb--done': step > 4 }" @click="goToStep(4)">
          Params
        </button>
      </template>
      <template v-if="step > 5">
        <span class="qb-crumb-sep">/</span>
        <button class="qb-crumb qb-crumb--done">
          {{ languageLabel }}
        </button>
      </template>
    </div>

    <!-- ── Step 1: Mode ─────────────────────────────────────────────────── -->
    <section v-if="step === 1" class="qb-section">
      <h2 class="qb-step-title">
        <span class="qb-step-number">1</span>
        What do you want?
      </h2>
      <div class="qb-cards qb-cards--2">
        <button class="qb-card" :class="{ 'qb-card--selected': dataMode === 'historical' }"
          @click="selectDataMode('historical')">
          <div class="qb-card-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
              stroke-linejoin="round">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
              <line x1="16" y1="2" x2="16" y2="6" />
              <line x1="8" y1="2" x2="8" y2="6" />
              <line x1="3" y1="10" x2="21" y2="10" />
            </svg>
          </div>
          <div class="qb-card-body">
            <div class="qb-card-title">Historical Data</div>
            <div class="qb-card-desc">Query past market data — trades, quotes, OHLC, Greeks, and more</div>
          </div>
        </button>
        <button class="qb-card" :class="{ 'qb-card--selected': dataMode === 'streaming' }"
          @click="selectDataMode('streaming')">
          <div class="qb-card-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
              stroke-linejoin="round">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
            </svg>
          </div>
          <div class="qb-card-body">
            <div class="qb-card-title">Real-Time Streaming</div>
            <div class="qb-card-desc">Subscribe to live market events via FPSS</div>
          </div>
        </button>
      </div>
    </section>

    <!-- ── Step 2: Asset ─────────────────────────────────────────────────── -->
    <section v-if="step === 2" class="qb-section">
      <h2 class="qb-step-title">
        <span class="qb-step-number">2</span>
        What asset?
      </h2>

      <!-- Streaming only supports Stock / Option -->
      <template v-if="dataMode === 'historical'">
        <div class="qb-cards qb-cards--3">
          <button class="qb-card" :class="{ 'qb-card--selected': assetType === 'stock' }"
            @click="selectAssetType('stock')">
            <div class="qb-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                stroke-linejoin="round">
                <polyline points="22 7 13.5 15.5 8.5 10.5 2 17" />
                <polyline points="16 7 22 7 22 13" />
              </svg>
            </div>
            <div class="qb-card-body">
              <div class="qb-card-title">Stock</div>
              <div class="qb-card-desc">Equities, e.g. AAPL, TSLA</div>
            </div>
          </button>
          <button class="qb-card" :class="{ 'qb-card--selected': assetType === 'option' }"
            @click="selectAssetType('option')">
            <div class="qb-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                stroke-linejoin="round">
                <circle cx="12" cy="12" r="10" />
                <path d="M9 9h.01M15 9h.01M9.5 15a3.5 3.5 0 0 0 5 0" />
              </svg>
            </div>
            <div class="qb-card-body">
              <div class="qb-card-title">Option</div>
              <div class="qb-card-desc">Calls & puts, e.g. SPY 450C 2024-06-21</div>
            </div>
          </button>
          <button class="qb-card" :class="{ 'qb-card--selected': assetType === 'index' }"
            @click="selectAssetType('index')">
            <div class="qb-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                stroke-linejoin="round">
                <line x1="18" y1="20" x2="18" y2="10" />
                <line x1="12" y1="20" x2="12" y2="4" />
                <line x1="6" y1="20" x2="6" y2="14" />
              </svg>
            </div>
            <div class="qb-card-body">
              <div class="qb-card-title">Index</div>
              <div class="qb-card-desc">Market indices, e.g. SPX, VIX</div>
            </div>
          </button>
          <button class="qb-card" :class="{ 'qb-card--selected': assetType === 'calendar' }"
            @click="selectAssetType('calendar')">
            <div class="qb-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                stroke-linejoin="round">
                <rect x="3" y="4" width="18" height="18" rx="2" />
                <line x1="16" y1="2" x2="16" y2="6" />
                <line x1="8" y1="2" x2="8" y2="6" />
                <line x1="3" y1="10" x2="21" y2="10" />
                <line x1="8" y1="14" x2="8" y2="14" stroke-width="2.5" stroke-linecap="round" />
                <line x1="12" y1="14" x2="12" y2="14" stroke-width="2.5" stroke-linecap="round" />
              </svg>
            </div>
            <div class="qb-card-body">
              <div class="qb-card-title">Calendar</div>
              <div class="qb-card-desc">Market hours &amp; holidays</div>
            </div>
          </button>
          <button class="qb-card" :class="{ 'qb-card--selected': assetType === 'rate' }"
            @click="selectAssetType('rate')">
            <div class="qb-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                stroke-linejoin="round">
                <line x1="12" y1="1" x2="12" y2="23" />
                <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
              </svg>
            </div>
            <div class="qb-card-body">
              <div class="qb-card-title">Interest Rate</div>
              <div class="qb-card-desc">Risk-free rate (SOFR, etc.)</div>
            </div>
          </button>
        </div>
      </template>

      <template v-else>
        <div class="qb-cards qb-cards--2">
          <button class="qb-card" :class="{ 'qb-card--selected': assetType === 'stock' }"
            @click="selectAssetType('stock')">
            <div class="qb-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                stroke-linejoin="round">
                <polyline points="22 7 13.5 15.5 8.5 10.5 2 17" />
                <polyline points="16 7 22 7 22 13" />
              </svg>
            </div>
            <div class="qb-card-body">
              <div class="qb-card-title">Stock</div>
              <div class="qb-card-desc">Stream live equity quotes &amp; trades</div>
            </div>
          </button>
          <button class="qb-card" :class="{ 'qb-card--selected': assetType === 'option' }"
            @click="selectAssetType('option')">
            <div class="qb-card-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                stroke-linejoin="round">
                <circle cx="12" cy="12" r="10" />
                <path d="M9 9h.01M15 9h.01M9.5 15a3.5 3.5 0 0 0 5 0" />
              </svg>
            </div>
            <div class="qb-card-body">
              <div class="qb-card-title">Option</div>
              <div class="qb-card-desc">Stream live option contract quotes &amp; trades</div>
            </div>
          </button>
        </div>
      </template>
    </section>

    <!-- ── Step 3: Data type ──────────────────────────────────────────────── -->
    <section v-if="step === 3" class="qb-section">
      <h2 class="qb-step-title">
        <span class="qb-step-number">3</span>
        What data?
      </h2>
      <div class="qb-list">
        <button v-for="opt in dataTypeOptions" :key="opt.id" class="qb-list-item"
          :class="{ 'qb-list-item--selected': dataType === opt.id }" @click="selectDataType(opt.id)">
          <div class="qb-list-label">{{ opt.label }}</div>
          <div class="qb-list-desc">{{ opt.description }}</div>
        </button>
      </div>
    </section>

    <!-- ── Step 4: Parameters ────────────────────────────────────────────── -->
    <section v-if="step === 4" class="qb-section">
      <h2 class="qb-step-title">
        <span class="qb-step-number">4</span>
        Parameters
      </h2>
      <div class="qb-form">

        <!-- Symbol -->
        <div class="qb-field" v-if="needsSymbol">
          <label class="qb-label" for="qb-symbol">
            {{ isSnapshotMulti ? 'Symbol (comma-separated)' : 'Symbol / Ticker' }}
          </label>
          <input id="qb-symbol" class="qb-input" type="text" v-model="params.symbol"
            :placeholder="isSnapshotMulti ? 'AAPL,TSLA,MSFT' : 'AAPL'" />
          <span class="qb-hint">Case-insensitive. e.g. AAPL</span>
        </div>

        <!-- Rate symbol -->
        <div class="qb-field" v-if="needsRateSymbol">
          <label class="qb-label" for="qb-rate-symbol">Rate Symbol</label>
          <input id="qb-rate-symbol" class="qb-input" type="text" v-model="params.rate_symbol"
            placeholder="SOFR" />
        </div>

        <!-- Date range -->
        <div class="qb-field-row" v-if="needsDateRange">
          <div class="qb-field">
            <label class="qb-label" for="qb-start">Start Date</label>
            <input id="qb-start" class="qb-input" type="text" v-model="params.start_date"
              placeholder="YYYYMMDD" maxlength="8" />
          </div>
          <div class="qb-field">
            <label class="qb-label" for="qb-end">End Date</label>
            <input id="qb-end" class="qb-input" type="text" v-model="params.end_date"
              placeholder="YYYYMMDD" maxlength="8" />
          </div>
        </div>

        <!-- Single date -->
        <div class="qb-field" v-if="needsSingleDate">
          <label class="qb-label" for="qb-date">Date</label>
          <input id="qb-date" class="qb-input" type="text" v-model="params.date"
            placeholder="YYYYMMDD" maxlength="8" />
        </div>

        <!-- Interval -->
        <div class="qb-field" v-if="needsInterval">
          <label class="qb-label" for="qb-interval">Interval</label>
          <select id="qb-interval" class="qb-select" v-model="params.interval">
            <option v-for="opt in intervalOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>

        <!-- Option expiration -->
        <div class="qb-field" v-if="needsExpiration">
          <label class="qb-label" for="qb-exp">Expiration Date</label>
          <input id="qb-exp" class="qb-input" type="text" v-model="params.expiration"
            placeholder="YYYYMMDD" maxlength="8" />
        </div>

        <!-- Option contract for streaming -->
        <template v-if="needsOptionContractForStream">
          <div class="qb-field">
            <label class="qb-label" for="qb-opt-sym">Underlying Symbol</label>
            <input id="qb-opt-sym" class="qb-input" type="text" v-model="params.symbol" placeholder="SPY" />
          </div>
          <div class="qb-field">
            <label class="qb-label" for="qb-opt-exp">Expiration Date</label>
            <input id="qb-opt-exp" class="qb-input" type="text" v-model="params.expiration"
              placeholder="YYYYMMDD" maxlength="8" />
          </div>
        </template>

        <!-- Strike + Right (option params) -->
        <template v-if="needsOptionParams || needsOptionContractForStream">
          <div class="qb-field">
            <label class="qb-label" for="qb-strike">Strike Price</label>
            <input id="qb-strike" class="qb-input" type="number" v-model="params.strike"
              placeholder="450" step="0.5" />
          </div>
          <div class="qb-field">
            <label class="qb-label">Right</label>
            <div class="qb-radio-group">
              <label class="qb-radio">
                <input type="radio" name="right" value="C" v-model="params.right" />
                <span class="qb-radio-label">Call</span>
              </label>
              <label class="qb-radio">
                <input type="radio" name="right" value="P" v-model="params.right" />
                <span class="qb-radio-label">Put</span>
              </label>
            </div>
          </div>
        </template>

        <!-- At-time -->
        <div class="qb-field" v-if="needsAtTime">
          <label class="qb-label" for="qb-time">Time of Day (HHMMSS)</label>
          <input id="qb-time" class="qb-input" type="text" v-model="params.time"
            placeholder="093000" maxlength="6" />
          <span class="qb-hint">e.g. 093000 for 09:30:00 ET</span>
        </div>

        <!-- Year (calendar) -->
        <div class="qb-field" v-if="needsYear">
          <label class="qb-label" for="qb-year">Year</label>
          <input id="qb-year" class="qb-input" type="number" v-model="params.year"
            placeholder="2024" min="2004" max="2030" />
        </div>

      </div>

      <button class="qb-btn-primary" @click="step = 5">
        Continue to Language
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
          stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </button>
    </section>

    <!-- ── Step 5: Language ──────────────────────────────────────────────── -->
    <section v-if="step === 5" class="qb-section">
      <h2 class="qb-step-title">
        <span class="qb-step-number">5</span>
        Choose language
      </h2>
      <div class="qb-cards qb-cards--2">
        <button class="qb-card" :class="{ 'qb-card--selected': language === 'rust' }"
          @click="selectLanguage('rust')">
          <div class="qb-card-icon qb-card-icon--lang">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M11.9 0a12 12 0 1 0 .2 0zm.7 2.76.75 1.43.85-.83 1.48 3.84-1.94.77 1 2.53-1.68-.66-.72 1.82-1.64-4.17.83-.32-.93-2.37zM7.02 3.5l.83.32-.93 2.37.83.32-1.64 4.17-.72-1.82-1.68.66 1-2.53-1.94-.77 1.48-3.84.85.83.75-1.43zm3.53.13 2.4.57-.26 1.06 1.56.38-.76 3.1-1.36-.34.14-.57-1.4-.34-.12.49-1.36-.34.76-3.1 1.56.38-.26-1.06zM12 12.7a.3.3 0 1 1 0 .6.3.3 0 0 1 0-.6zm-4.92 1.7 1.22.5-.26.64-1.68-.68.02-.7 1.43.06-.4-.96.62-.26.27.65-.61.25-.16.39-.45.11zm9.84 0 .16.39-.45-.11-.61-.25.27-.65.62.26-.4.96 1.43-.06.02.7-1.68.68-.26-.64 1.22-.5-.32-.78z" />
            </svg>
          </div>
          <div class="qb-card-body">
            <div class="qb-card-title">Rust</div>
            <div class="qb-card-desc">Async/await with Tokio</div>
          </div>
        </button>
        <button class="qb-card" :class="{ 'qb-card--selected': language === 'python' }"
          @click="selectLanguage('python')">
          <div class="qb-card-icon qb-card-icon--lang">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M11.99 2C6.47 2 6.81 4.37 6.81 4.37l.01 2.45h5.28v.73H4.81S2 7.22 2 12.8s2.45 5.43 2.45 5.43H5.9v-2.61s-.07-2.45 2.41-2.45h4.16s2.33.04 2.33-2.25V6.29S15.18 2 11.99 2zm-1.37 1.33c.42 0 .76.34.76.76s-.34.76-.76.76-.76-.34-.76-.76.34-.76.76-.76z" />
              <path
                d="M12.01 22c5.52 0 5.18-2.37 5.18-2.37l-.01-2.45h-5.28v-.73h7.29S22 16.78 22 11.2s-2.45-5.43-2.45-5.43h-1.45v2.61s.07 2.45-2.41 2.45H11.53s-2.33-.04-2.33 2.25v3.63S8.82 22 12.01 22zm1.37-1.33c-.42 0-.76-.34-.76-.76s.34-.76.76-.76.76.34.76.76-.34.76-.76.76z" />
            </svg>
          </div>
          <div class="qb-card-body">
            <div class="qb-card-title">Python</div>
            <div class="qb-card-desc">Synchronous API with f-string output</div>
          </div>
        </button>
      </div>
    </section>

    <!-- ── Step 6: Output ────────────────────────────────────────────────── -->
    <section v-if="step === 6 && generatedCode" class="qb-section">
      <h2 class="qb-step-title">
        <span class="qb-step-number">6</span>
        Generated code
      </h2>
      <div class="qb-output">
        <div class="qb-output-header">
          <span class="qb-output-lang">{{ codeLanguageLabel }}</span>
          <button class="qb-copy-btn" @click="copyCode" :class="{ 'qb-copy-btn--copied': copied }">
            <template v-if="!copied">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                stroke-linejoin="round">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
              Copy
            </template>
            <template v-else>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"
                stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12" />
              </svg>
              Copied!
            </template>
          </button>
        </div>
        <pre class="qb-pre"><code :class="`language-${codeLanguageLabel}`">{{ generatedCode }}</code></pre>
      </div>

      <button class="qb-btn-reset" @click="reset">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
          stroke-linejoin="round">
          <polyline points="1 4 1 10 7 10" />
          <path d="M3.51 15a9 9 0 1 0 .49-3.5" />
        </svg>
        Start a new query
      </button>
    </section>

  </div>
</template>

<style scoped>
/* ─── Root ─────────────────────────────────────────────────────────────── */
.qb-root {
  max-width: 760px;
  margin: 0 auto;
  padding: 8px 0 48px;
  font-family: var(--vp-font-family-base);
}

/* ─── Trail / breadcrumb ─────────────────────────────────────────────── */
.qb-trail {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
  margin-bottom: 28px;
  font-size: 13px;
}

.qb-crumb {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--vp-c-text-3);
  padding: 2px 4px;
  border-radius: 4px;
  transition: color 0.15s;
  font-size: 13px;
  font-family: var(--vp-font-family-base);
}

.qb-crumb:hover {
  color: var(--vp-c-brand-1);
}

.qb-crumb--active {
  color: var(--vp-c-text-3);
  font-weight: 500;
}

.qb-crumb--done {
  color: var(--vp-c-brand-1);
  font-weight: 600;
}

.qb-crumb-sep {
  color: var(--vp-c-text-3);
  user-select: none;
}

/* ─── Section ────────────────────────────────────────────────────────── */
.qb-section {
  animation: qb-fade-in 0.2s ease;
}

@keyframes qb-fade-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* ─── Step title ─────────────────────────────────────────────────────── */
.qb-step-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 18px;
  font-weight: 700;
  color: var(--vp-c-text-1);
  margin: 0 0 20px;
  padding: 0;
  border: none;
  letter-spacing: -0.01em;
}

.qb-step-number {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--vp-c-brand-1);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
}

/* ─── Cards (mode / asset / language) ───────────────────────────────── */
.qb-cards {
  display: grid;
  gap: 12px;
}

.qb-cards--2 {
  grid-template-columns: repeat(2, 1fr);
}

.qb-cards--3 {
  grid-template-columns: repeat(3, 1fr);
}

@media (max-width: 540px) {

  .qb-cards--2,
  .qb-cards--3 {
    grid-template-columns: 1fr;
  }
}

.qb-card {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 16px 18px;
  border: 1.5px solid var(--vp-c-divider);
  border-radius: 10px;
  background: var(--vp-c-bg-soft);
  cursor: pointer;
  text-align: left;
  transition: border-color 0.15s, box-shadow 0.15s, background 0.15s;
  font-family: var(--vp-font-family-base);
}

.qb-card:hover {
  border-color: var(--vp-c-brand-1);
  background: var(--vp-c-bg);
  box-shadow: 0 2px 12px var(--vp-c-brand-soft);
}

.qb-card--selected {
  border-color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.qb-card-icon {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: var(--vp-c-bg);
  border: 1px solid var(--vp-c-divider);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--vp-c-brand-1);
  margin-top: 1px;
}

.qb-card-icon svg {
  width: 18px;
  height: 18px;
}

.qb-card-icon--lang {
  background: var(--vp-c-bg);
}

.qb-card--selected .qb-card-icon {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
  color: #fff;
}

.qb-card-body {
  flex: 1;
  min-width: 0;
}

.qb-card-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--vp-c-text-1);
  margin-bottom: 3px;
  line-height: 1.3;
}

.qb-card-desc {
  font-size: 13px;
  color: var(--vp-c-text-2);
  line-height: 1.5;
}

/* ─── List (data type) ───────────────────────────────────────────────── */
.qb-list {
  display: flex;
  flex-direction: column;
  gap: 0;
  border: 1.5px solid var(--vp-c-divider);
  border-radius: 10px;
  overflow: hidden;
}

.qb-list-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 12px 16px;
  border: none;
  border-bottom: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
  cursor: pointer;
  text-align: left;
  transition: background 0.12s;
  font-family: var(--vp-font-family-base);
}

.qb-list-item:last-child {
  border-bottom: none;
}

.qb-list-item:hover {
  background: var(--vp-c-bg-soft);
}

.qb-list-item--selected {
  background: var(--vp-c-brand-soft);
  border-left: 3px solid var(--vp-c-brand-1);
}

.qb-list-item--selected .qb-list-label {
  color: var(--vp-c-brand-1);
}

.qb-list-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--vp-c-text-1);
  margin-bottom: 2px;
}

.qb-list-desc {
  font-size: 12.5px;
  color: var(--vp-c-text-2);
  line-height: 1.4;
}

/* ─── Form ───────────────────────────────────────────────────────────── */
.qb-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
  margin-bottom: 24px;
  padding: 20px;
  border: 1.5px solid var(--vp-c-divider);
  border-radius: 10px;
  background: var(--vp-c-bg-soft);
}

.qb-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.qb-field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

@media (max-width: 480px) {
  .qb-field-row {
    grid-template-columns: 1fr;
  }
}

.qb-label {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--vp-c-text-2);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.qb-input,
.qb-select {
  height: 38px;
  padding: 0 12px;
  border: 1.5px solid var(--vp-c-divider);
  border-radius: 7px;
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  font-size: 14px;
  font-family: var(--vp-font-family-mono);
  transition: border-color 0.15s;
  outline: none;
  width: 100%;
  box-sizing: border-box;
}

.qb-input:focus,
.qb-select:focus {
  border-color: var(--vp-c-brand-1);
  box-shadow: 0 0 0 3px var(--vp-c-brand-soft);
}

.qb-select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%238b949e' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  background-size: 16px;
  padding-right: 34px;
  cursor: pointer;
}

.qb-hint {
  font-size: 11.5px;
  color: var(--vp-c-text-3);
}

.qb-radio-group {
  display: flex;
  gap: 16px;
  padding: 8px 0;
}

.qb-radio {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.qb-radio input[type="radio"] {
  accent-color: var(--vp-c-brand-1);
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.qb-radio-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--vp-c-text-1);
}

/* ─── Continue button ────────────────────────────────────────────────── */
.qb-btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 8px;
  border: none;
  background: var(--vp-c-brand-1);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s, transform 0.1s;
  font-family: var(--vp-font-family-base);
}

.qb-btn-primary:hover {
  background: var(--vp-c-brand-2);
}

.qb-btn-primary:active {
  transform: scale(0.98);
}

.qb-btn-primary svg {
  width: 16px;
  height: 16px;
}

/* ─── Reset button ───────────────────────────────────────────────────── */
.qb-btn-reset {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 9px 18px;
  border-radius: 8px;
  border: 1.5px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-2);
  font-size: 13.5px;
  font-weight: 500;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
  font-family: var(--vp-font-family-base);
  margin-top: 16px;
}

.qb-btn-reset:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.qb-btn-reset svg {
  width: 15px;
  height: 15px;
}

/* ─── Code output ────────────────────────────────────────────────────── */
.qb-output {
  border: 1.5px solid var(--vp-c-divider);
  border-radius: 10px;
  overflow: hidden;
  background: var(--vp-c-bg-soft);
}

.qb-output-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
}

.qb-output-lang {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--vp-c-text-3);
}

.qb-copy-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-2);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  font-family: var(--vp-font-family-base);
}

.qb-copy-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.qb-copy-btn--copied {
  border-color: #22c55e;
  color: #22c55e;
  background: rgba(34, 197, 94, 0.08);
}

.qb-copy-btn svg {
  width: 13px;
  height: 13px;
}

.qb-pre {
  margin: 0;
  padding: 20px;
  overflow-x: auto;
  background: transparent;
  border: none;
  border-radius: 0;
}

.qb-pre code {
  font-family: var(--vp-font-family-mono);
  font-size: 13.5px;
  line-height: 1.7;
  color: var(--vp-c-text-1);
  background: transparent;
  border: none;
  padding: 0;
  font-weight: normal;
  white-space: pre;
  display: block;
}
</style>
