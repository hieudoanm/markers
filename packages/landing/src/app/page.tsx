import type { Metadata } from 'next';
import Image from 'next/image';
import Link from 'next/link';
import type { FC, JSX } from 'react';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import CTA from '@/components/molecules/CTA';
import {
  PiRobot,
  PiFileText,
  PiGlobe,
  PiPalette,
  PiLightning,
  PiLayout,
  PiFlowArrow,
} from 'react-icons/pi';

export const metadata: Metadata = {
  title: { absolute: 'markers — static site generator for Markdown docs' },
};

const LOGO =
  process.env.NODE_ENV === 'production' ? '/markers/logo.png' : '/logo.png';
const GITHUB = 'https://github.com/hieudoanm/markers';

interface CompareRow {
  feature: string;
  markers: string;
  docsify: string;
}

const COMPARISON: CompareRow[] = [
  {
    feature: 'Rendering',
    markers: 'Ahead of time → static HTML',
    docsify: 'In the browser at runtime',
  },
  {
    feature: 'JavaScript required',
    markers: 'None for content',
    docsify: 'Required',
  },
  {
    feature: 'Hosting',
    markers: 'Any static host',
    docsify: 'Needs correct SPA fallback',
  },
  {
    feature: 'Offline / cacheable',
    markers: 'Fully',
    docsify: 'Only the shell',
  },
  {
    feature: 'Search engines',
    markers: 'Full HTML + sitemap',
    docsify: 'Limited',
  },
  {
    feature: 'Runtime',
    markers: 'One Rust binary',
    docsify: 'Node at build time',
  },
];

type TokKind =
  'prompt' | 'cmd' | 'flag' | 'str' | 'key' | 'url' | 'dim' | 'ok' | 'err';
interface Tok {
  t: string;
  c?: TokKind;
}

const TK: Record<TokKind, string> = {
  prompt: 'text-accent select-none',
  cmd: 'font-semibold text-sky-300',
  flag: 'text-amber-400',
  str: 'text-green-300',
  key: 'text-violet-300',
  url: 'text-cyan-300',
  dim: 'text-slate-400',
  ok: 'text-gray-200',
  err: 'text-red-400',
};

interface TerminalProps {
  lines: Tok[][];
}

const Terminal: FC<TerminalProps> = ({ lines }) => (
  <div className="mockup-code">
    {lines.map((line, i) => {
      const prompt = line[0]?.c === 'prompt' ? line[0].t.trim() : null;
      const tokens = prompt ? line.slice(1) : line;
      return (
        <pre key={i} {...(prompt ? { 'data-prefix': prompt } : {})}>
          <code>
            {tokens.map((tok, j) => (
              <span key={j} className={TK[tok.c ?? 'ok']}>
                {tok.t}
              </span>
            ))}
          </code>
        </pre>
      );
    })}
  </div>
);

const FEATURES: Feature[] = [
  {
    icon: PiLightning,
    title: 'Zero configuration',
    body: 'Point markers at a folder of Markdown and it builds a complete site — sidebar navigation, clean trailing-slash URLs, and an "On this page" outline are generated for you.',
  },
  {
    icon: PiFileText,
    title: 'GitHub-flavoured Markdown',
    body: 'Tables, task lists, footnotes, alerts, fenced code with per-language accents, and {#custom-ids} — rendered by pulldown-cmark, no JavaScript required.',
  },
  {
    icon: PiPalette,
    title: 'Eight themes, light & dark',
    body: 'A compiled md.css with light, dark and six colour palettes. Theme follows the OS by default, with a one-click toggle persisted in localStorage.',
  },
  {
    icon: PiFlowArrow,
    title: 'Built-in page graph',
    body: 'Every build emits graph.html — an interactive map of how your pages link to each other — plus graph.json and Graphviz DOT for further analysis.',
  },
  {
    icon: PiLayout,
    title: 'Landing pages too',
    body: 'markers landing builds a whole single-page site — marketing, waitlist, docs hub, linktree — from one landing.yaml, with 12 layouts and 64 themes.',
  },
  {
    icon: PiGlobe,
    title: 'Wikipedia importer',
    body: 'markers wiki convert turns any Wikipedia article into clean local Markdown, rewriting internal links to filenames so whole wikis become browsable sites.',
  },
  {
    icon: PiRobot,
    title: 'MCP server',
    body: 'markers-mcp exposes build, serve and site inspection as MCP tools — let your editor or agent work with the docs directly.',
  },
];

const LandingPage = (): JSX.Element => (
  <>
    <main id="top">
      {/* hero */}
      <Hero
        title={
          <>
            Markdown docs,{' '}
            <span className="text-primary">compiled to static HTML</span> — in
            one Rust binary
          </>
        }
        description={
          <>
            markers turns a folder of{' '}
            <code className="border-base-300 bg-base-200 rounded border px-1.5 py-0.5 font-mono text-sm">
              .md
            </code>{' '}
            files into a fast, self-contained static site: navigation, themes,
            clean URLs and a page graph included. No Node, no runtime, no
            server.
          </>
        }
        primary={{ label: 'Get started', href: '/docs/getting-started/' }}
        secondary={{ label: 'View on GitHub', href: GITHUB, external: true }}
        stats={[
          { value: '1', label: 'Rust binary' },
          { value: '8', label: 'built-in themes' },
          { value: '0', label: 'JavaScript required' },
        ]}>
        <Terminal
          lines={[
            [
              { t: '$ ', c: 'prompt' },
              { t: 'markers', c: 'cmd' },
              { t: ' build ', c: 'dim' },
              { t: './docs', c: 'url' },
              { t: ' --output ', c: 'dim' },
              { t: 'dist', c: 'flag' },
            ],
            [
              { t: '  ', c: 'dim' },
              { t: 'Built 13 pages and 8 assets into dist in 6.8ms', c: 'ok' },
            ],
            [
              { t: '$ ', c: 'prompt' },
              { t: 'markers', c: 'cmd' },
              { t: ' serve ', c: 'dim' },
              { t: 'dist', c: 'url' },
            ],
            [
              { t: '➜ ', c: 'prompt' },
              { t: 'Serving dist at ', c: 'dim' },
              { t: 'http://localhost:8080', c: 'url' },
            ],
          ]}
        />
      </Hero>

      {/* features */}
      <Features
        title="Everything a docs site needs."
        description="Sensible defaults, real output — each feature is one flag or zero flags away."
        features={FEATURES}
      />

      {/* comparison */}
      <section id="compare" className="bg-gray-50 py-20">
        <div className="mx-auto w-full max-w-6xl px-5">
          <div className="mx-auto mb-12 max-w-2xl text-center">
            <h2 className="m-0 mb-2 text-2xl leading-tight font-bold tracking-tight text-gray-900 sm:text-3xl lg:text-4xl">
              markers vs docsify.
            </h2>
            <p className="mt-3 text-lg text-gray-500">
              Same Markdown-in, docs-out idea — opposite rendering model.
            </p>
          </div>
          <div className="overflow-x-auto rounded-2xl border border-gray-200">
            <table className="w-full border-collapse text-sm [&_tbody_tr:last-child>td]:border-b-0">
              <thead>
                <tr>
                  <th className="w-44 border-b border-gray-200 bg-gray-50 px-5 py-3.5 text-left align-bottom font-semibold" />
                  <th className="border-b border-l border-gray-200 bg-gray-50 px-5 py-3.5 text-left align-bottom font-semibold">
                    docsify
                  </th>
                  <th className="bg-primary/10 border-b border-l border-gray-200 px-5 py-3.5 text-left align-bottom">
                    <span className="text-primary flex items-center gap-2 font-bold">
                      markers
                      <span className="badge badge-primary uppercase">
                        pick
                      </span>
                    </span>
                  </th>
                </tr>
              </thead>
              <tbody>
                {COMPARISON.map((row) => (
                  <tr key={row.feature} className="group">
                    <td className="border-b border-gray-200 px-5 py-3.5 align-top font-medium text-gray-500">
                      {row.feature}
                    </td>
                    <td className="border-b border-l border-gray-200 px-5 py-3.5 align-top text-gray-500">
                      {row.docsify}
                    </td>
                    <td className="bg-primary/5 group-hover:bg-primary/10 border-b border-l border-gray-200 px-5 py-3.5 align-top font-medium text-gray-900">
                      {row.markers}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* quickstart */}
      <section id="quickstart" className="py-20">
        <div className="mx-auto w-full max-w-6xl px-5">
          <div className="mx-auto mb-12 max-w-2xl text-center">
            <h2 className="m-0 mb-2 text-2xl leading-tight font-bold tracking-tight text-gray-900 sm:text-3xl lg:text-4xl">
              Up and running in under a minute.
            </h2>
            <p className="mt-3 text-lg text-gray-500">
              Install the binary, point it at a Markdown folder, serve the
              result — or scaffold a landing page with{' '}
              <code className="rounded border border-gray-200 bg-gray-50 px-1.5 py-0.5 font-mono text-sm">
                markers landing new
              </code>
              .
            </p>
          </div>
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
            <Terminal
              lines={[
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'curl', c: 'cmd' },
                  {
                    t: ' -fsSL https://raw.githubusercontent.com/hieudoanm/markers/main/scripts/install.sh',
                    c: 'url',
                  },
                  { t: ' | sh', c: 'dim' },
                ],
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'cargo', c: 'cmd' },
                  { t: ' install ', c: 'dim' },
                  { t: '--path', c: 'flag' },
                  { t: ' crates/markers-cli', c: 'dim' },
                ],
                [
                  { t: '➜ ', c: 'prompt' },
                  { t: 'installed markers ', c: 'dim' },
                  { t: '~/.local/bin', c: 'ok' },
                ],
              ]}
            />
            <Terminal
              lines={[
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'markers', c: 'cmd' },
                  { t: ' build ', c: 'dim' },
                  { t: './docs', c: 'url' },
                  { t: ' --output ', c: 'dim' },
                  { t: 'dist', c: 'flag' },
                ],
                [
                  { t: '  ', c: 'dim' },
                  {
                    t: 'Built 13 pages and 8 assets into dist in 6.8ms',
                    c: 'ok',
                  },
                ],
                [
                  { t: '$ ', c: 'prompt' },
                  { t: 'markers', c: 'cmd' },
                  { t: ' serve ', c: 'dim' },
                  { t: 'dist', c: 'url' },
                  { t: ' --port ', c: 'dim' },
                  { t: '8080', c: 'flag' },
                ],
                [
                  { t: '➜ ', c: 'prompt' },
                  { t: 'Serving dist at ', c: 'dim' },
                  { t: 'http://localhost:8080', c: 'url' },
                ],
              ]}
            />
          </div>
          <p className="mt-8 text-center text-sm text-gray-500">
            Or generate a landing page:{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1.5 py-0.5 font-mono text-sm">
              markers landing new --type product
            </code>{' '}
            →{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1.5 py-0.5 font-mono text-sm">
              markers landing build -o public/
            </code>
            . Full walkthrough in{' '}
            <Link href="/docs/guide/getting-started/" className="text-primary">
              getting started
            </Link>
            .
          </p>
        </div>
      </section>

      <CTA
        title="Ship your docs anywhere."
        description={
          <>
            One binary, one{' '}
            <code className="rounded border border-gray-200 bg-gray-50 px-1 py-px font-mono text-sm">
              markers build
            </code>
            , one static folder — ready for GitHub Pages, any CDN, or a plain
            web server.
          </>
        }
        primary={{ label: 'Download', href: '/downloads/' }}
        secondary={{ label: 'View Docs', href: '/docs/' }}
      />
    </main>
  </>
);

export default LandingPage;
