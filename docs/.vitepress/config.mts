import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'NemCSS',
  description: 'A design-token-driven CSS utility generator',
  head: [['link', { rel: 'icon', href: '/favicon.ico' }]],
  themeConfig: {
    nav: [
      { text: 'Guide', link: '/guide/introduction' },
      { text: 'Integrations', link: '/integrations/cli' },
      { text: 'Examples', link: '/examples/vanilla' },
      { text: 'Editor', link: '/editor/' },
    ],

    sidebar: {
      '/guide/': [
        { text: 'Introduction', link: '/guide/introduction' },
        { text: 'Getting Started', link: '/guide/getting-started' },
        { text: 'Design Tokens', link: '/guide/design-tokens' },
        { text: 'Framework Support', link: '/guide/framework-support' },
        { text: 'Configuration', link: '/guide/configuration' },
      ],
      '/integrations/': [
        { text: 'CLI', link: '/integrations/cli' },
        { text: 'Vite', link: '/integrations/vite' },
        { text: 'PostCSS', link: '/integrations/postcss' },
      ],
      '/examples/': [
        { text: 'Vanilla HTML', link: '/examples/vanilla' },
        { text: 'Astro', link: '/examples/astro' },
        { text: 'React SPA (Vite)', link: '/examples/react-spa' },
      ],
      '/editor/': [
        { text: 'Editor Support', link: '/editor/' },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/liv7c/nemcss' },
    ],
  },
})
