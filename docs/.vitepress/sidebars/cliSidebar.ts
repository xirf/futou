import type { DefaultTheme } from 'vitepress';

export const cliSidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'CLI Reference',
    collapsed: false,
    items: [
      { text: 'Overview', link: '/cli/' },
      { text: 'Install & Uninstall', link: '/cli/install' },
      { text: 'Activate & Deactivate', link: '/cli/activate' },
      { text: 'Catalogue & Status', link: '/cli/catalogue' },
    ]
  }
];
