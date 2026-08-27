'use client';

import { use, useEffect, useId, useState } from 'react';
import type { CSSProperties } from 'react';
import { useTheme } from 'next-themes';

type MermaidProps = {
  chart: string;
  minWidth?: CSSProperties['minWidth'];
};

export function Mermaid({ chart, minWidth }: MermaidProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) return;
  return <MermaidContent chart={chart} minWidth={minWidth} />;
}

const cache = new Map<string, Promise<unknown>>();

function cachePromise<T>(key: string, setPromise: () => Promise<T>): Promise<T> {
  const cached = cache.get(key);
  if (cached) return cached as Promise<T>;

  const promise = setPromise();
  cache.set(key, promise);
  return promise;
}

function MermaidContent({ chart, minWidth }: MermaidProps) {
  const id = useId();
  const { resolvedTheme } = useTheme();
  const { default: mermaid } = use(cachePromise('mermaid', () => import('mermaid')));

  mermaid.initialize({
    startOnLoad: false,
    securityLevel: 'loose',
    fontFamily: 'inherit',
    themeCSS: 'margin: 1.5rem auto 0;',
    theme: resolvedTheme === 'dark' ? 'dark' : 'default',
  });

  const { svg, bindFunctions } = use(
    cachePromise(`${chart}-${resolvedTheme}`, () => {
      return mermaid.render(id, chart.replaceAll('\\n', '\n'));
    }),
  );

  const mermaidStyle = {
    overflowX: 'auto',
    '--mermaid-min-width':
      typeof minWidth === 'number' ? `${minWidth}px` : minWidth,
  } as CSSProperties & { '--mermaid-min-width'?: string };

  return (
    <div
      className="mermaid-diagram"
      ref={(container) => {
        if (container) bindFunctions?.(container);
      }}
      style={mermaidStyle}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}
