import Link from 'next/link';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { Braces, SquarePen } from 'lucide-react';
import { baseOptions } from '@/lib/layout.shared';

export default function NotFound() {
  return (
    <HomeLayout
      {...baseOptions()}
      links={[
        {
          type: 'menu',
          text: 'Documentation',
          items: [
            {
              icon: <Braces />,
              text: 'Framework',
              description: 'The core framework',
              url: '/docs/framework/introduction/what-is-lokey',
            },
            {
              icon: <SquarePen />,
              text: 'Editor',
              description: 'GUI for configuring Lokey devices',
              url: '/docs/editor',
            },
          ],
        },
      ]}
    >
      <main className="flex flex-1 flex-col items-center justify-center px-6 py-24 text-center">
        <p className="text-sm font-medium uppercase tracking-[0.2em] text-fd-muted-foreground">404</p>
        <h1 className="mt-4 text-3xl font-semibold tracking-tight text-fd-foreground">Page not found</h1>
        <p className="mt-4 text-fd-muted-foreground">
          The page you are looking for does not exist.
        </p>
        <Link href="/" className="mt-8 rounded-full bg-fd-primary px-4 py-2 text-fd-primary-foreground">
          Return home
        </Link>
      </main>
    </HomeLayout>
  );
}
