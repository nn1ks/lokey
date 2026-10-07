import { Braces, SquarePen } from 'lucide-react';
import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export const homeLinks: NonNullable<BaseLayoutProps['links']> = [
  {
    type: 'menu',
    text: 'Documentation',
    url: '/docs/framework/introduction/getting-started',
    items: [
      {
        type: 'main',
        icon: <Braces />,
        text: 'Framework',
        description: 'The core framework',
        url: '/docs/framework/introduction/getting-started',
      },
      {
        icon: <SquarePen />,
        text: 'Editor',
        description: 'GUI for configuring Lokey devices',
        url: '/docs/editor/getting-started',
      },
    ],
  },
  {
    type: 'main',
    text: 'Blog',
    url: '/blog',
  },
];
