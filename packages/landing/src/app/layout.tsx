import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/molecules/Footer';
import Navbar from '@/components/molecules/Navbar';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

const GITHUB = 'https://github.com/hieudoanm/markers';
const LOGO =
  process.env.NODE_ENV === 'production' ? '/markers/logo.png' : '/logo.png';
const NAV_LINKS = [
  { label: 'Features', href: '#features' },
  { label: 'vs docsify', href: '#compare' },
  { label: 'Quickstart', href: '#quickstart' },
  { label: 'Docs', href: '/docs/' },
  { label: 'Downloads', href: '/downloads/' },
];

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io/markers/'),
  title: {
    default: 'markers — static site generator for Markdown docs',
    template: '%s — markers',
  },
  description:
    'A single Rust binary that turns a folder of Markdown into a fast, self-contained static site — clean URLs, navigation, themes, a page graph, and a Wikipedia importer. No Node, no runtime.',
  keywords: [
    'static site generator',
    'Markdown',
    'Rust',
    'documentation site',
    'Wikipedia import',
    'MCP server',
    'no Node.js',
  ],
  authors: [{ name: 'hieudoan', url: 'https://github.com/hieudoanm/markers' }],
  category: 'software',
  openGraph: {
    siteName: 'markers',
    type: 'website',
    title: 'markers — static site generator for Markdown docs',
    description:
      'Markdown folders → static HTML in one Rust binary. Clean URLs, themes, graphs, Wikipedia import, MCP server.',
    url: 'https://hieudoanm.github.io/markers/',
    images: ['/og.png'],
  },
  twitter: { card: 'summary_large_image', images: ['/og.png'] },
};

const JSON_LD = {
  '@context': 'https://schema.org',
  '@graph': [
    {
      '@type': 'WebSite',
      name: 'markers',
      url: 'https://hieudoanm.github.io/markers/',
      description:
        'A single Rust binary that turns a folder of Markdown into a fast, self-contained static site — clean URLs, navigation, themes, a page graph, and a Wikipedia importer. No Node, no runtime.',
    },
    {
      '@type': 'SoftwareApplication',
      name: 'markers',
      applicationCategory: 'DeveloperApplication',
      description:
        'A single Rust binary that turns a folder of Markdown into a fast, self-contained static site — clean URLs, navigation, themes, a page graph, and a Wikipedia importer. No Node, no runtime.',
      url: 'https://hieudoanm.github.io/markers/',
      codeRepository: 'https://github.com/hieudoanm/markers',
      offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
    },
  ],
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => (
  <html
    lang="en"
    data-theme="markers"
    className={`scroll-smooth antialiased motion-reduce:scroll-auto ${inter.className}`}>
    <body className="bg-base-100 text-base-content m-0 leading-relaxed">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(JSON_LD) }}
      />
      <Navbar
        name="markers"
        logoSrc={LOGO}
        links={NAV_LINKS}
        githubUrl={GITHUB}
      />
      {children}
      <Footer
        name="markers"
        tagline="Static sites from Markdown, in one Rust binary · MIT"
        githubUrl={GITHUB}
      />
      
    </body>
  </html>
);

export default RootLayout;
