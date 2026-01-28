"use client";
import TransactionRow, { TransactionData } from "./TransactionRow";

// interface TransactionListProps {
//   transactions: TransactionData[];
// }

export function TransactionList({ transactions }: { transactions: TransactionData[] }) {
  return (
    <div className="w-full flex flex-col h-full overflow-hidden ">
      <div className="overflow-x-auto flex-1 transaction-list-scrollable">
        <table className="w-full text-left border-collapse md:min-w-[800px] lg:min-w-[1000px] table-fixed">
          <colgroup className="hidden md:table-column-group">
            <col style={{ width: '18%' }} />
            <col style={{ width: '18%' }} />
            <col style={{ width: '12%' }} />
            <col style={{ width: '18%' }} />
            <col style={{ width: '10%' }} />
            <col style={{ width: '12%' }} />
            <col style={{ width: '12%' }} />
          </colgroup>
          <colgroup className="md:hidden">
            <col style={{ width: '100%' }} />
          </colgroup>
          <thead className="sticky top-0 z-30 hidden md:table-header-group">
            <tr className="border-b border-white/10">
              <th className="pl-6 lg:pl-10 py-4 text-[#A0A5BA] text-sm lg:text-base font-normal tracking-wider bg-[#0A0B0F] z-30">
                Group Name
              </th>
              <th className="px-4 lg:px-10 py-4 text-[#A0A5BA] text-sm lg:text-base font-normal tracking-wider bg-[#0A0B0F] z-30">
                Group Address
              </th>
              <th className="px-4 lg:px-10 py-4 text-[#A0A5BA] text-sm lg:text-base font-normal tracking-wider bg-[#0A0B0F] z-30">
                Total Amount
              </th>
              <th className="px-4 lg:px-10 py-4 text-[#A0A5BA] text-sm lg:text-base font-normal tracking-wider bg-[#0A0B0F] z-30">
                Amount Breakdown
              </th>
              <th className="px-4 lg:px-10 py-4 text-[#A0A5BA] text-sm lg:text-base font-normal tracking-wider text-left bg-[#0A0B0F] z-30">
                Members
              </th>
              <th className="px-4 lg:px-10 py-4 text-[#A0A5BA] text-sm lg:text-base font-normal tracking-wider bg-[#0A0B0F] z-30">
                Time
              </th>
              <th className="px-4 lg:px-10 py-4 text-[#A0A5BA] text-sm lg:text-base font-normal tracking-wider text-left bg-[#0A0B0F] z-30">
                Date
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-white/5">
            {transactions.map((tx, index) => (
              <TransactionRow key={tx.id} transaction={tx} isFirstTwoRows={index < 2} />
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

