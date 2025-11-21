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
      { text: 'Docs', link: '/introduction/what-is-lokey', activeMatch: '/introduction/|/concepts/|/guides/|/keyboard/|/mouse/|/encoder/|/midi-controller/|/examples/' }
    ],

    sidebar: [
      {
        text: 'Introduction',
        collapsed: false,
        items: [
          { text: 'What is Lokey?', link: '/introduction/what-is-lokey' },
          { text: 'Supported Hardware', link: '/introduction/supported-hardware' },
          { text: 'Getting Started', link: '/introduction/getting-started' },
          { text: 'API Documentation', link: '/introduction/api-documentation' },
        ]
      },
      {
        text: 'Concepts',
        collapsed: false,
        items: [
          { text: 'Devices', link: '/concepts/devices' },
          { text: 'Components', link: '/concepts/components' },
          { text: 'MCUs', link: '/concepts/mcus' },
          { text: 'External Transports', link: '/concepts/external-transports' },
          { text: 'Internal Transports', link: '/concepts/internal-transports' },
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
          { text: 'Implementing a USB Class', link: '/guides/implementing-a-usb-class' },
          { text: 'Implementing a BLE Service', link: '/guides/implementing-a-ble-service' },
          { text: 'Adding Support for a MCU', link: '/guides/adding-support-for-a-mcu' },
        ]
      },
      {
        text: 'Keyboard',
        collapsed: false,
        items: [
          { text: 'Introduction', link: '/keyboard/introduction' },
          { text: 'Layout', link: '/keyboard/layout' },
          { text: 'Actions', link: '/keyboard/actions' },
          { text: 'Scanning', link: '/keyboard/scanning' },
          { text: 'Debouncing', link: '/keyboard/debouncing' },
          { text: 'Key Overrides', link: '/keyboard/key-overrides' },
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
      {
        text: 'Examples',
        collapsed: false,
        items: [
          { text: 'TODO' },
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/nn1ks/lokey' }
    ]
  },
})
