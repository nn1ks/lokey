import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { baseOptions } from '@/lib/layout.shared';
import { Braces, SquarePen } from 'lucide-react';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Lokey',
};

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <HomeLayout
      {...baseOptions()}
      links={[
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
      ]}
    >
      {children}
    </HomeLayout>
  );
}
