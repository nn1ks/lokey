import Image from 'next/image';
import Link from 'next/link';
import { DynamicCodeBlock } from 'fumadocs-ui/components/dynamic-codeblock';
import { AnimatedTitle } from './animated-title';
import styles from './home-page.module.css';

const features = [
  {
    label: 'Toolchain',
    title: 'Powered by Rust & Embassy',
    description: (
      <>
        Built in{' '}
        <a href="https://www.rust-lang.org/" target="_blank" rel="noopener noreferrer">
          Rust
        </a>{' '}
        for speed and safety, leveraging the async embedded framework{' '}
        <a href="https://embassy.dev/" target="_blank" rel="noopener noreferrer">
          Embassy
        </a>
        .
      </>
    ),
    icon: 'chip',
  },
  {
    label: 'Architecture',
    title: 'Modular & Extensible',
    description:
      'Structured as a highly modular system, making it possible to easily add functionality and use the framework for a variety of input devices.',
    icon: 'grid',
  },
  {
    label: 'Developer Experience',
    title: 'User-Friendly API',
    description:
      'An API that is designed to be intuitive and fast, with compile-time checks that prevent mistakes from reaching your device.',
    icon: 'code',
  },
  {
    label: 'Composability',
    title: 'Multi-Part Device Support',
    description: 'Includes first-class support for devices that consist of multiple parts (e.g. split keyboards).',
    icon: 'parts',
  },
  {
    label: 'Connectivity',
    title: 'Wireless Support',
    description:
      'Supports connecting devices to the host and to each other via Bluetooth Low Energy, with power-efficient components for battery-powered use.',
    icon: 'wireless',
  },
  {
    label: 'License',
    title: 'Open Source',
    description: 'Licensed under either of Apache License, Version 2.0 or MIT License at your option.',
    icon: 'open-source',
  },
] as const;

type FeatureIconName = (typeof features)[number]['icon'];

function FeatureIcon({ name }: { name: FeatureIconName }) {
  const commonProps = {
    viewBox: '0 0 160 160',
    fill: 'none',
    'aria-hidden': true,
  } as const;

  switch (name) {
    case 'chip':
      return (
        <svg {...commonProps}>
          <rect x="32" y="32" width="96" height="96" rx="8" />
          <path
            d="M56 31 L56 13 M104 31 L104 13 M56 129 L56 147 M104 129 L104 147 M31 56 L13 56 M31 104 L13 104 M129 56 L147 56 M129 104 L147 104"
            strokeLinecap="round"
          />
        </svg>
      );
    case 'grid':
      return (
        <svg {...commonProps}>
          <rect x="9" y="9" width="62" height="62" rx="9" />
          <rect x="89" y="9" width="62" height="62" rx="9" />
          <rect x="9" y="89" width="62" height="62" rx="9" />
          <rect x="89" y="89" width="62" height="62" rx="9" />
        </svg>
      );
    case 'code':
      return (
        <svg {...commonProps}>
          <path
            d="M60 34 L16 80 L60 126 M100 34 L144 80 L100 126"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      );
    case 'parts':
      return (
        <svg {...commonProps}>
          <circle cx="30" cy="80" r="28" />
          <circle cx="130" cy="80" r="28" />
          <line x1="58" y1="80" x2="102" y2="80" />
        </svg>
      );
    case 'wireless':
      return (
        <svg {...commonProps}>
          <path
            d="M44 44 L116 116 L80 152 L80 8 L116 44 L44 116"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      );
    case 'open-source':
      return (
        <svg {...commonProps}>
          <path
            d="M56 150 A 70 70 0 1 1 104 150 L 90 110 A 28 28 0 1 0 70 110 L 56 150"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      );
  }
}

const exampleCode = `#![no_main]
#![no_std]

use embassy_executor::Spawner;
use lokey::Context;
use lokey_keyboard::{Key, Scanner, layout};
use lokey_my_device::{MyDevice, MyTransports, MyState};

#[lokey::device]
async fn main(context: Context<MyDevice, MyTransports, MyState>, spawner: Spawner) {
    let scanner = Scanner::new();
    let layout = layout!([Key::A, Key::B, Key::C]);
    context.enable_all((scanner, layout)).await;
}`;

export default function HomePage() {
  return (
    <div className={`${styles.homePage} mx-auto w-full max-w-6xl px-6 py-16 sm:py-24`}>
      <section className="grid grid-cols-1 items-center gap-12 text-left lg:gap-16 xl:gap-48 lg:grid-cols-[minmax(0,1fr)_280px]">
        <div>
          <h1 className={`${styles.heading} max-w-4xl text-fd-foreground`}>
            <span className="heading-name">
              <AnimatedTitle>Lokey</AnimatedTitle>
            </span>
            <span className="heading-text"> — A firmware framework for input devices</span>
          </h1>
          <p className="mt-4 max-w-2xl text-lg leading-8 font-medium text-fd-muted-foreground sm:text-2xl sm:mt-6">
            Easily create modular firmware for keyboards, mice, MIDI controllers, and more.
          </p>
        </div>
        <div className="order-first flex justify-center lg:order-0">
          <Image
            src="/logo-black-thin.svg"
            alt="Lokey logo"
            width={180}
            height={180}
            className="h-28 w-28 dark:hidden lg:h-45 lg:w-45"
          />
          <Image
            src="/logo-white-thin.svg"
            alt=""
            width={180}
            height={180}
            className="hidden h-28 w-28 dark:block lg:h-45 lg:w-45"
          />
        </div>
      </section>

      <div className={`${styles.headingButtons} mt-6 flex flex-wrap justify-start gap-3 text-sm sm:text-base sm:mt-8`}>
        <Link
          href="/docs/framework/introduction/getting-started"
          className={`${styles.primaryButton} rounded-lg px-4 py-2 font-medium transition-colors text-fd-primary-foreground`}
        >
          Get Started
        </Link>
        <Link
          href="/docs/framework/introduction/api-documentation"
          className={`${styles.secondaryButton} rounded-lg border px-5 py-2 font-medium transition-colors`}
        >
          API Documentation
        </Link>
        <a
          href="https://github.com/nn1ks/lokey"
          target="_blank"
          rel="noopener noreferrer"
          className={`${styles.secondaryButton} rounded-lg border px-5 py-2 font-medium transition-colors`}
        >
          Source Code
        </a>
      </div>

      <section className="mt-20" aria-label="Code example">
        <DynamicCodeBlock
          lang="rust"
          code={exampleCode}
          codeblock={{ title: 'main.rs' }}
          options={{
            themes: {
              light: 'gruvbox-light-hard',
              dark: 'gruvbox-dark-hard',
            },
          }}
        />
      </section>

      <section className={styles.featuresSection} aria-label="Features">
        {features.map((feature, index) => (
          <article key={feature.title} className={`${styles.featureItem} ${index % 2 === 1 ? styles.featureItemReverse : ''}`}>
            <div className={styles.featureIllustration}>
              <FeatureIcon name={feature.icon} />
            </div>
            <div className={styles.featureContent}>
              <span className={styles.featureLabel}>{feature.label}</span>
              <h2 className={styles.featureTitle}>{feature.title}</h2>
              <p className={styles.featureDesc}>{feature.description}</p>
            </div>
          </article>
        ))}
      </section>


    </div>
  );
}
