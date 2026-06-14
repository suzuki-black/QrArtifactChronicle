// QRの内容から「生成年」を抽出（docs/02 2.2 / docs/07 Q1）。
// 明示的な日時を持つ構造のみ対象。推測しない。取れなければ null。
// 複数候補は最も新しい年（=補正最小、保守的）を採用。

const PLAUSIBLE_MIN = 1970; // unix epoch 起点
const PLAUSIBLE_MAX = 2099;

function plausible(year: number): boolean {
  return Number.isInteger(year) && year >= PLAUSIBLE_MIN && year <= PLAUSIBLE_MAX;
}

function yearFromUnix(n: number): number | null {
  // 10桁=秒, 13桁=ミリ秒
  const ms = n < 1e12 ? n * 1000 : n;
  const y = new Date(ms).getUTCFullYear();
  return plausible(y) ? y : null;
}

/**
 * @param text QR の復号テキスト（バイナリQRは null を渡す → 常に null）
 * @returns 抽出できた年、なければ null
 */
export function extractYear(text: string | null): number | null {
  if (!text) return null;
  const candidates: number[] = [];

  // 1. vCard REV / iCal DTSTAMP・DTSTART: ラベル直後の YYYY を採用
  for (const m of text.matchAll(/(?:^|\n)\s*(?:REV|DTSTAMP|DTSTART)[:;][^\n]*?(\d{4})(\d{2}|-\d{2})/gi)) {
    const y = Number(m[1]);
    if (plausible(y)) candidates.push(y);
  }

  // 2. URL内の UNIX タイムスタンプ（10/13桁の独立した数値）
  if (/^https?:\/\//i.test(text)) {
    for (const m of text.matchAll(/(?<!\d)(\d{13}|\d{10})(?!\d)/g)) {
      const y = yearFromUnix(Number(m[1]));
      if (y !== null) candidates.push(y);
    }
  }

  // 3. ISO 8601 日付（YYYY-MM-DD、前後が数字でない）
  for (const m of text.matchAll(/(?<!\d)(\d{4})-(\d{2})-(\d{2})(?!\d)/g)) {
    const y = Number(m[1]);
    const mo = Number(m[2]);
    const d = Number(m[3]);
    if (plausible(y) && mo >= 1 && mo <= 12 && d >= 1 && d <= 31) candidates.push(y);
  }

  if (candidates.length === 0) return null;
  return Math.max(...candidates); // 最も新しい年（保守的）
}
