import TransactionRow, { TransactionData } from "./TransactionRow";

export function TransactionList({ transactions }: { transactions: TransactionData[] }) {
  return (
    <div className="divide-y divide-white/5">
      {transactions.map((t) => (
        <TransactionRow key={t.id} transaction={t} />
      ))}
    </div>
  );
}
