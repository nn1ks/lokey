'use client';

import { useEffect, useRef } from 'react';

const SVG_PER_LETTER = [
  ['/title-svg/l1.svg', '/title-svg/l2.svg', '/title-svg/l3.svg'],
  ['/title-svg/o1.svg', '/title-svg/o2.svg', '/title-svg/o3.svg'],
  ['/title-svg/k1.svg', '/title-svg/k2.svg', '/title-svg/k3.svg'],
  ['/title-svg/e1.svg', '/title-svg/e2.svg', '/title-svg/e3.svg'],
  ['/title-svg/y1.svg', '/title-svg/y2.svg', '/title-svg/y3.svg'],
];

const NUM_CHANGES = 3;
const INTERVAL_MS = 800;

export function AnimatedTitle({ children }: { children: string }) {
  const titleRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const title = titleRef.current;
    if (!title) return;
    const element = title;

    let cancelled = false;
    let interval: number | undefined;

    async function start() {
      const text = element.textContent || '';
      const svgCache: string[][] = [];

      for (let i = 0; i < text.length; i++) {
        const letterSvgs = await Promise.all(
          SVG_PER_LETTER[i].map(async (path) => {
            const response = await fetch(path);
            let svgText = await response.text();
            const width = svgText.match(/width="([\d.]+)px"/);
            const height = svgText.match(/height="([\d.]+)px"/);

            if (width && height) {
              svgText = svgText
                .replace(/width="[\d.]+px"/, '')
                .replace(/height="[\d.]+px"/, '')
                .replace('<svg', `<svg viewBox="0 0 ${width[1]} ${height[1]}"`);
            }

            return svgText;
          }),
        );
        svgCache.push(letterSvgs);
      }

      if (cancelled || svgCache.length !== text.length) return;

      element.style.width = 'max-content';
      const totalWidth = element.getBoundingClientRect().width;
      element.style.position = 'relative';
      element.style.visibility = 'hidden';

      const overlay = document.createElement('span');
      overlay.style.position = 'absolute';
      overlay.style.inset = '0';
      overlay.style.visibility = 'visible';

      const containers: { element: HTMLSpanElement; svgs: string[] }[] = [];
      const step = totalWidth / text.length;

      for (let i = 0; i < text.length; i++) {
        const container = document.createElement('span');
        container.className = 'title-svg';
        container.style.position = 'absolute';
        container.style.height = '1.2em';
        container.style.left = `${i * step}px`;
        container.style.top = '50%';
        container.style.transform = 'translateY(-50%)';
        container.style.display = 'flex';
        container.style.alignItems = 'center';
        container.style.justifyContent = 'center';
        container.innerHTML = svgCache[i][0];
        overlay.appendChild(container);
        containers.push({ element: container, svgs: svgCache[i] });
      }

      element.appendChild(overlay);

      let ticks = 0;
      let svgIndex = 0;

      const cycle = () => {
        if (ticks >= NUM_CHANGES) {
          if (interval !== undefined) window.clearInterval(interval);
          overlay.remove();
          element.style.visibility = '';
          element.style.position = '';
          element.style.width = '';
          return;
        }

        containers.forEach(({ element, svgs }) => {
          element.innerHTML = svgs[svgIndex];
        });
        svgIndex++;
        ticks++;
      };

      cycle();
      interval = window.setInterval(cycle, INTERVAL_MS);
    }

    void start();

    return () => {
      cancelled = true;
      if (interval !== undefined) window.clearInterval(interval);
      element.querySelector('.title-svg')?.parentElement?.remove();
      element.style.visibility = '';
      element.style.position = '';
      element.style.width = '';
    };
  }, []);

  return <span ref={titleRef}>{children}</span>;
}
