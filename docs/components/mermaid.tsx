'use client';

import { useEffect, useId, useState } from 'react';
import { useTheme } from 'next-themes';
import mermaid from 'mermaid';

export function Mermaid({ chart }: { chart: string }) {
  const id = useId();
  const { resolvedTheme } = useTheme();
  const [svg, setSvg] = useState<string>('');

  useEffect(() => {
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: 'loose',
      fontFamily: 'inherit',
      themeCSS: 'margin: 1.5rem auto 0;',
      theme: resolvedTheme === 'dark' ? 'dark' : 'default',
    });

    const renderChart = async () => {
      try {
        // ID needs to be HTML-safe and unique
        const safeId = 'mermaid-' + id.replace(/:/g, '');
        const result = await mermaid.render(safeId, chart.replaceAll('\\n', '\n'));
        setSvg(result.svg);
      } catch (err) {
        console.error('Mermaid parsing error:', err);
      }
    };

    renderChart();
  }, [chart, resolvedTheme, id]);

  if (!svg) return null;

  return (
    <div
      className="my-6 overflow-x-auto"
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}
