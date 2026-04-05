import { defineConfig } from 'vitepress'
import footnote from 'markdown-it-footnote'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'Lokey',
  description: 'A firmware framework for input devices',

  head: [
    ['link', { rel: "apple-touch-icon", sizes: "180x180", href: "/favicon/apple-touch-icon.png" }],
    ['link', { rel: "icon", type: "image/png", sizes: "32x32", href: "/favicon/favicon-32x32.png" }],
    ['link', { rel: "icon", type: "image/png", sizes: "16x16", href: "/favicon/favicon-16x16.png" }],
    ['link', { rel: "manifest", href: "/favicon/site.webmanifest" }],
  ],

  sitemap: {
    hostname: "https://lokey.rs",
  },

  cleanUrls: true,
  rewrites(id) {
    return id.replace(/^pages\/(.+)/, '$1')
  },

  lastUpdated: true,

  markdown: {
    config: (md) => {
      md.use(footnote)
    }
  },

  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config

    logo: '/logo.svg',

    search: {
      provider: 'local',
    },

    footer: {
      copyright: 'Copyright © 2025 Niklas Sauter',
    },

    editLink: {
      pattern: 'https://github.com/nn1ks/lokey/edit/master/docs/:path'
    },

    nav: [
      { text: 'Home', link: '/' },
      { text: 'Docs', link: '/introduction/what-is-lokey', activeMatch: '/introduction/|/concepts/|/guides/|/keyboard/|/mouse/|/midi-controller/' }
    ],

    sidebar: [
      {
        text: 'Introduction',
        collapsed: false,
        items: [
          { text: 'What is Lokey?', link: '/introduction/what-is-lokey' },
          { text: 'Getting Started', link: '/introduction/getting-started' },
          { text: 'Supported Hardware', link: '/introduction/supported-hardware' },
          { text: 'API Documentation', link: '/introduction/api-documentation' },
          { text: 'Troubleshooting', link: '/introduction/troubleshooting' },
        ]
      },
      {
        text: 'Concepts',
        collapsed: false,
        items: [
          { text: 'Devices', link: '/concepts/devices' },
          { text: 'Components', link: '/concepts/components' },
          { text: 'MCUs', link: '/concepts/mcus' },
          { text: 'Storage', link: '/concepts/storage' },
          { text: 'External Transports', link: '/concepts/external-transports' },
          { text: 'External Channel', link: '/concepts/external-channel' },
          { text: 'Internal Transports', link: '/concepts/internal-transports' },
          { text: 'Internal Channel', link: '/concepts/internal-channel' },
          { text: 'State', link: '/concepts/state' },
          { text: 'Context', link: '/concepts/context' },
        ]
      },
      {
        text: 'Guides',
        collapsed: false,
        items: [
          { text: 'Writing a Custom Component', link: '/guides/writing-a-custom-component' },
          { text: 'Adding Support for a Device', link: '/guides/adding-support-for-a-device' },
          { text: 'Adding Support for an MCU', link: '/guides/adding-support-for-an-mcu' },
          { text: 'Flashing Firmware', link: '/guides/flashing-firmware' },
        ]
      },
      {
        text: 'Keyboard',
        collapsed: false,
        items: [
          { text: 'Introduction', link: '/keyboard/introduction' },
          { text: 'Scanning', link: '/keyboard/scanning' },
          { text: 'Actions', link: '/keyboard/actions' },
          { text: 'Layout', link: '/keyboard/layout' },
          { text: 'Comparison with Alternatives', link: '/keyboard/comparison-with-alternatives' },
        ]
      },
      {
        text: 'Mouse',
        collapsed: false,
        items: [
          { text: 'Introduction', link: '/mouse/introduction' },
        ]
      },
      {
        text: 'MIDI Controller',
        collapsed: false,
        items: [
          { text: 'Introduction', link: '/midi-controller/introduction' },
        ]
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/nn1ks/lokey' }
    ]
  },
})
