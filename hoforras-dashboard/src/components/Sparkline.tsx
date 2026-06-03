// A tiny inline SVG sparkline (FR-9.3) — no external charting dependency.

interface SparklineProps {
  values: number[];
  width?: number;
  height?: number;
}

export function Sparkline({ values, width = 96, height = 24 }: SparklineProps) {
  if (values.length === 0) {
    return <svg width={width} height={height} role="img" aria-label="sparkline empty" />;
  }
  const max = Math.max(...values, 1);
  const step = values.length > 1 ? width / (values.length - 1) : width;
  const points = values
    .map((v, i) => `${(i * step).toFixed(1)},${(height - (v / max) * height).toFixed(1)}`)
    .join(" ");
  return (
    <svg width={width} height={height} role="img" aria-label="24h demand sparkline">
      <polyline points={points} fill="none" stroke="currentColor" strokeWidth={1.5} />
    </svg>
  );
}
