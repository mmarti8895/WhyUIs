import "../styles/brand-fire.css";

interface Props {
  size?: "sm" | "md" | "lg";
  framed?: boolean;
  className?: string;
}

export default function BrandFireIcon({ size = "md", framed = false, className = "" }: Props) {
  const classes = ["brand-fire", `brand-fire-${size}`, framed ? "brand-fire-framed" : "", className]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={classes} aria-hidden="true">
      <span className="brand-fire-glow" />
      <span className="brand-fire-outer" />
      <span className="brand-fire-inner" />
      <span className="brand-fire-core" />
      <span className="brand-fire-log brand-fire-log-left" />
      <span className="brand-fire-log brand-fire-log-center" />
      <span className="brand-fire-log brand-fire-log-right" />
      <span className="brand-fire-ember brand-fire-ember-1" />
      <span className="brand-fire-ember brand-fire-ember-2" />
      <span className="brand-fire-ember brand-fire-ember-3" />
    </div>
  );
}
