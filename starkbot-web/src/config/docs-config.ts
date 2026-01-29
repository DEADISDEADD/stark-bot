export interface DocItem {
  label: string
  to: string
}

export interface DocSection {
  title: string
  items: DocItem[]
}

export const docsConfig = {
  sections: [
    {
      title: 'Getting Started',
      items: [
        { label: 'Overview', to: '/docs' },
        { label: 'Getting Started', to: '/docs/getting-started' },
        { label: 'Architecture', to: '/docs/architecture' },
      ],
    },
    {
      title: 'Core Concepts',
      items: [
        { label: 'Tools', to: '/docs/tools' },
        { label: 'Skills', to: '/docs/skills' },
        { label: 'Channels', to: '/docs/channels' },
        { label: 'Telegram', to: '/docs/telegram' },
        { label: 'Memories', to: '/docs/memories' },
      ],
    },
    {
      title: 'Configuration',
      items: [
        { label: 'Configuration', to: '/docs/configuration' },
        { label: 'Scheduling', to: '/docs/scheduling' },
      ],
    },
    {
      title: 'Reference',
      items: [
        { label: 'API Reference', to: '/docs/api' },
      ],
    },
  ] as DocSection[],
}

export default docsConfig
