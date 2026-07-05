import type { DefaultTheme } from 'vitepress';

export const guideSidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'Introduction',
    collapsed: false,
    items: [
      { text: 'What is Futou?', link: '/guide/getting-started' },
      { text: 'Installation', link: '/guide/installation' },
      { text: 'Architecture', link: '/guide/architecture' },
    ]
  },
  {
    text: 'Development',
    collapsed: true,
    items: [
      { text: 'Building from Source', link: '/guide/building' },
    ]
  }
];
