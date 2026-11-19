import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'markers Desktop — PDF to Markdown',
  description:
    'Convert PDF articles to TEI XML and Markdown locally via GROBID — no upload, no cloud.'
};

const RootLayout: FC<{ children: ReactNode }> = ({ children }) => (
  <html lang="en" data-theme="markers" className={inter.className}>
    <body className="bg-base-200 text-base-content flex min-h-screen flex-col">
      {children}
    </body>
  </html>
);

export default RootLayout;
