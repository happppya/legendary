// Self-hosted type: IBM Plex Sans (variable) for UI + IBM Plex Mono for
// identifiers/counts. Fontsource bundles woff2 subsets into dist/ - no
// runtime font fetch, no Google Fonts <link>.
import '@fontsource-variable/ibm-plex-sans/wght.css'
import '@fontsource/ibm-plex-mono/400.css'
import '@fontsource/ibm-plex-mono/500.css'
import '@fontsource/ibm-plex-mono/600.css'

import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

// Resolve the theme before the first paint so there is no flash of the wrong
// mode. A stored choice wins; otherwise follow the system preference.
const root = document.documentElement
let theme = 'dark'
try {
  const stored = localStorage.getItem('legendary-theme')
  if (stored === 'light' || stored === 'dark') theme = stored
  else if (window.matchMedia('(prefers-color-scheme: light)').matches) theme = 'light'
} catch {
  /* storage unavailable - keep the dark default */
}
root.dataset.theme = theme

createApp(App).mount('#app')
