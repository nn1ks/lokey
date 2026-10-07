import { HomeLayout } from 'fumadocs-ui/layouts/home';
import { baseOptions } from '@/lib/layout.shared';
import type { Metadata } from 'next';
import { homeLinks } from '@/lib/home-links';

export const metadata: Metadata = {
  title: 'Lokey',
};

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <HomeLayout
      {...baseOptions()}
      links={homeLinks}
    >
      {children}
    </HomeLayout>
  );
}
