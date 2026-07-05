import { fileURLToPath, URL } from 'node:url'
import UnoCSS from 'unocss/vite';
import { defineConfig, type DefaultTheme } from 'vitepress';
import { guideSidebar } from './sidebars/guideSidebar';
import { cliSidebar } from './sidebars/cliSidebar';
import { runtimesSidebar } from './sidebars/runtimesSidebar';

export default defineConfig({
  title: 'Futou',
  description: 'Windows Environment Manager — install, activate, and manage runtimes',
  outDir: './dist',
  lastUpdated: true,
  ignoreDeadLinks: false,
  lang: 'en-US',
  head: [
    ['link', { rel: 'icon', href: '/molniya.svg' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    [
      'link',
      {
        rel: 'stylesheet',
        href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&family=JetBrains+Mono:wght@400;500&display=swap',
      },
    ],
  ],
  vite: {
    plugins: [UnoCSS()],
    resolve: {
      alias: [
        {
          find: /^.*\/VPDocAsideOutline\.vue$/,
          replacement: fileURLToPath(
            new URL('./theme/components/DocOutline.vue', import.meta.url)
          )
        },
        {
          find: /^.*\/VPSwitchAppearance\.vue$/,
          replacement: fileURLToPath(
            new URL('./theme/components/ThemeSwitcher.vue', import.meta.url)
          )
        },
        {
          find: /^.*\/VPSidebarItem\.vue$/,
          replacement: fileURLToPath(
            new URL('./theme/components/VPSidebarItem.vue', import.meta.url)
          )
        }
      ]
    }
  },
  markdown: {
    theme: {
      light: 'catppuccin-latte',
      dark: 'catppuccin-macchiato',
    },
    lineNumbers: true,
  },
  themeConfig: {
    logo: '/molniya.svg',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'CLI', link: '/cli/' },
      { text: 'Runtimes', link: '/runtimes/' },
    ],

    sidebar: {
      '/guide/': guideSidebar,
      '/cli/': cliSidebar,
      '/runtimes/': runtimesSidebar,
    } satisfies DefaultTheme.Sidebar,

    outline: {
      level: [2, 3],
      label: 'On this page'
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/xirf/futou' }
    ],

    editLink: {
      pattern: 'https://github.com/xirf/futou/edit/master/docs/:path',
      text: 'Suggest changes to this page'
    },
  }
})
