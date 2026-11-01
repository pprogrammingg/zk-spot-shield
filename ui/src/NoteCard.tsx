import type { ReactNode } from "react";

type NoteCardProps = {
  kind: "spend" | "change";
  amount: string;
  settled: boolean;
  locked?: boolean;
  children?: ReactNode;
};

export function NoteCard({
  kind,
  amount,
  settled,
  locked = false,
  children,
}: NoteCardProps) {
  const spent = kind === "spend" && settled && !locked;
  const preview = kind === "change" && !settled;
  const shine = !locked && (kind === "change" ? settled : !spent);
  return (
    <article
      className={`note-card ${kind}${locked ? " locked" : ""}${spent ? " spent" : ""}${preview ? " preview" : ""}`}
    >
      {shine && <span className="glare" aria-hidden="true" />}
      <p className="note-kicker">Private mint A</p>
      <p className="note-amount">{amount}</p>
      {locked ? (
        <p className="note-lock">Locked for Spending ....</p>
      ) : null}
      {children}
    </article>
  );
}
