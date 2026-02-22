/**
 * Stellar utility functions for interacting with the Horizon API
 */

const HORIZON_URL = "https://horizon.stellar.org";

interface AccountDetails {
  id: string;
  balances: Array<{
    balance: string;
    asset_type: string;
    asset_code?: string;
    asset_issuer?: string;
  }>;
}

/**
 * Fetches the account details for a given Stellar public key
 * @param publicKey - The Stellar public key
 * @returns Account details from Horizon API
 */
export async function getAccountDetails(
  publicKey: string,
): Promise<AccountDetails | null> {
  try {
    const response = await fetch(`${HORIZON_URL}/accounts/${publicKey}`);
    if (!response.ok) {
      return null;
    }
    return response.json();
  } catch (error) {
    console.error("Error fetching account details:", error);
    return null;
  }
}

/**
 * Fetches the native (XLM) balance for a given Stellar public key
 * @param publicKey - The Stellar public key
 * @returns XLM balance as a string, or null if not found
 */
export async function getNativeBalance(
  publicKey: string,
): Promise<string | null> {
  const accountDetails = await getAccountDetails(publicKey);
  if (!accountDetails) return null;

  const nativeBalance = accountDetails.balances.find(
    (balance) => balance.asset_type === "native",
  );

  return nativeBalance ? nativeBalance.balance : null;
}

/**
 * Abbreviates a Stellar public key
 * @param publicKey - The full Stellar public key
 * @param startChars - Number of characters to show at the start (default: 4)
 * @param endChars - Number of characters to show at the end (default: 4)
 * @returns Abbreviated key (e.g., GABCD...WXYZ)
 */
export function abbreviatePublicKey(
  publicKey: string,
  startChars: number = 4,
  endChars: number = 4,
): string {
  if (!publicKey || publicKey.length < startChars + endChars) {
    return publicKey;
  }
  return `${publicKey.substring(0, startChars)}...${publicKey.substring(
    publicKey.length - endChars,
  )}`;
}

/**
 * Formats a balance to a readable string with limited decimal places
 * @param balance - The balance as a string
 * @param decimals - Number of decimal places (default: 2)
 * @returns Formatted balance string
 */
export function formatBalance(balance: string, decimals: number = 2): string {
  try {
    const num = parseFloat(balance);
    if (isNaN(num)) return "0.00";
    return num.toFixed(decimals);
  } catch {
    return "0.00";
  }
}
