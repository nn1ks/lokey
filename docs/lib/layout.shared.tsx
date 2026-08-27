import Image from 'next/image';
import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { appName, gitConfig } from './shared';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="inline-flex items-center gap-2">
          <Image
            src="/logo-black.svg"
            alt=""
            width={22}
            height={22}
            className="dark:hidden"
          />
          <Image
            src="/logo-white.svg"
            alt=""
            width={22}
            height={22}
            className="hidden dark:block"
          />
          <span>{appName}</span>
        </span>
      ),
    },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  };
}
