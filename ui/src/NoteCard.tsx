import type { ReactNode } from "react";

type NoteCardProps = {
  kind: "spend" | "change";
  amount: string;
  subtitle: string;
  settled: boolean;
  children?: ReactNode;
};

export function NoteCard({
  kind,
  amount,
  subtitle,
  settled,
  children,
}: NoteCardProps) {
  const spent = kind === "spend" && settled;
  const preview = kind === "change" && !settled;
  const shine = kind === "change" ? settled : !spent;
  return (
    <article
      className={`note-card ${kind}${spent ? " spent" : ""}${preview ? " preview" : ""}`}
    >
      {shine && <span className="glare" aria-hidden="true" />}
      <p className="note-kicker">
        {kind === "spend"
          ? spent
            ? "Note balance · mint A · spent"
            : "Note balance · private mint A"
          : "Change note · private mint A"}
      </p>
      <p className="note-amount">{amount}</p>
      <p className="note-sub">{subtitle}</p>
      {children}
    </article>
  );
}
