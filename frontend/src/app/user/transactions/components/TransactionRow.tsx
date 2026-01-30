export type TransactionData = {
  id: string;
  groupName: string;
  groupAddress?: string;
  totalAmount?: string;
  members?: number;
  time?: string;
  date?: string;
  tokens?: Array<{ name: string; ticker?: string; amount?: string; icon?: string; value?: string }>;
};

export default function TransactionRow({ transaction }: { transaction: TransactionData }) {
  return (
    <div className="flex items-center gap-4 p-3 border-b border-white/5 hover:bg-white/2">
      <div className="flex-1">
        <div className="text-sm font-semibold">{transaction.groupName}</div>
        <div className="text-xs text-gray-400">{transaction.groupAddress}</div>
      </div>
      <div className="text-sm">{transaction.totalAmount ?? "-"}</div>
    </div>
  );
}
