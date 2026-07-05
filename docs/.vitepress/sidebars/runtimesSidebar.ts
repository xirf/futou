import type { DefaultTheme } from 'vitepress';

export const runtimesSidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'Runtimes',
    collapsed: false,
    items: [
      { text: 'Overview', link: '/runtimes/' },
      { text: 'PHP', link: '/runtimes/php' },
      { text: 'Node.js', link: '/runtimes/nodejs' },
      { text: 'Python', link: '/runtimes/python' },
      { text: 'Deno', link: '/runtimes/deno' },
      { text: 'MariaDB', link: '/runtimes/mariadb' },
      { text: 'PostgreSQL', link: '/runtimes/postgresql' },
      { text: 'Apache', link: '/runtimes/apache' },
      { text: 'Nginx', link: '/runtimes/nginx' },
    ]
  }
];
