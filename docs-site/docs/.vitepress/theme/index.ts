import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import LanguageSelector from './components/LanguageSelector.vue'
import TierBadge from './components/TierBadge.vue'
import QueryBuilder from './components/QueryBuilder.vue'
import './style.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('LanguageSelector', LanguageSelector)
    app.component('TierBadge', TierBadge)
    app.component('QueryBuilder', QueryBuilder)
  },
} satisfies Theme
