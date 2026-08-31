import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'openbim-mvd',
  description: 'Pure-Rust typed mvdXML 1.1 model, codec, rules, and validation',
  lang: 'en-US',
  base: '/mvd/',
  cleanUrls: true,
  lastUpdated: true,
  sitemap: { hostname: 'https://openbimrs.github.io/mvd/' },
  head: [
    ['meta', { name: 'theme-color', content: '#2456a6' }],
    ['meta', { name: 'robots', content: 'index,follow' }],
  ],
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'openbim-mvd',
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Capabilities', link: '/capabilities' },
      { text: 'Architecture', link: '/architecture' },
      { text: 'API', link: 'https://docs.rs/openbim-mvd' },
      { text: 'GitHub', link: 'https://github.com/openbimrs/mvd' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [
            { text: 'Getting started', link: '/guide/getting-started' },
            { text: 'Rules and validation', link: '/guide/rules-and-validation' },
          ],
        },
      ],
      '/': [
        {
          text: 'Project',
          items: [
            { text: 'Capabilities', link: '/capabilities' },
            { text: 'Architecture', link: '/architecture' },
            { text: 'Security', link: '/security' },
            { text: 'Standards boundary', link: '/standards-boundary' },
            { text: 'Changelog', link: '/project/changelog' },
          ],
        },
      ],
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/openbimrs/mvd' }],
    editLink: {
      pattern: 'https://github.com/openbimrs/mvd/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },
    search: { provider: 'local' },
    footer: {
      message: 'Implementation licensed under AGPL-3.0-or-later. Official standards material is not redistributed.',
      copyright: 'Copyright © 2026 openbimrs contributors',
    },
  },
})
