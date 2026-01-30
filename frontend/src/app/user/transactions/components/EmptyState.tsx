export default function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-12">
      <div className="text-lg font-semibold">No transactions yet</div>
      <p className="text-sm text-gray-400">Your transactions will appear here once you start using Paymesh.</p>
    </div>
  );
}
