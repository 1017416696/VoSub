/**
 * 字符级别文本差异对比工具
 */

export type DiffType = 'equal' | 'delete' | 'insert'

export interface DiffSegment {
  type: DiffType
  text: string
  // 用于选择：true = 使用新文本，false = 使用原文本
  useNew: boolean
}

/**
 * 计算两个字符串的最长公共子序列 (LCS)
 */
function lcs(a: string, b: string): string {
  const m = a.length
  const n = b.length
  const dp: number[][] = Array(m + 1).fill(null).map(() => Array(n + 1).fill(0))

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (a[i - 1] === b[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1])
      }
    }
  }

  // 回溯找出 LCS
  let result = ''
  let i = m, j = n
  while (i > 0 && j > 0) {
    if (a[i - 1] === b[j - 1]) {
      result = a[i - 1] + result
      i--
      j--
    } else if (dp[i - 1][j] > dp[i][j - 1]) {
      i--
    } else {
      j--
    }
  }
  return result
}

/**
 * 基于 LCS 计算差异片段
 */
export function computeDiff(original: string, corrected: string): DiffSegment[] {
  const common = lcs(original, corrected)
  const segments: DiffSegment[] = []

  let oi = 0, ci = 0, li = 0

  while (li < common.length) {
    const char = common[li]

    // 收集原文中被删除的字符
    let deleted = ''
    while (oi < original.length && original[oi] !== char) {
      deleted += original[oi]
      oi++
    }

    // 收集新文本中被插入的字符
    let inserted = ''
    while (ci < corrected.length && corrected[ci] !== char) {
      inserted += corrected[ci]
      ci++
    }

    // 如果有删除和插入，合并为一个替换片段
    if (deleted || inserted) {
      segments.push({
        type: deleted && inserted ? 'delete' : (deleted ? 'delete' : 'insert'),
        text: deleted || inserted,
        useNew: true // 默认使用新文本
      })
      if (deleted && inserted) {
        // 替换情况：删除后紧跟插入
        segments[segments.length - 1] = {
          type: 'delete',
          text: deleted,
          useNew: true
        }
        segments.push({
          type: 'insert',
          text: inserted,
          useNew: true
        })
      }
    }

    // 添加相同的字符
    let equal = ''
    while (li < common.length && oi < original.length && ci < corrected.length &&
           original[oi] === common[li] && corrected[ci] === common[li]) {
      equal += common[li]
      oi++
      ci++
      li++
    }
    if (equal) {
      segments.push({ type: 'equal', text: equal, useNew: false })
    }
  }

  // 处理剩余字符
  if (oi < original.length) {
    segments.push({ type: 'delete', text: original.slice(oi), useNew: true })
  }
  if (ci < corrected.length) {
    segments.push({ type: 'insert', text: corrected.slice(ci), useNew: true })
  }

  // 合并相邻的相同类型片段，并将 delete+insert 配对
  return mergeAndPairSegments(segments)
}

/**
 * 合并相邻片段并配对 delete/insert
 */
function mergeAndPairSegments(segments: DiffSegment[]): DiffSegment[] {
  if (segments.length === 0) return segments

  const result: DiffSegment[] = []
  let i = 0

  while (i < segments.length) {
    const seg = segments[i]

    // 合并连续的相同类型
    if (result.length > 0 && result[result.length - 1].type === seg.type) {
      result[result.length - 1].text += seg.text
    } else {
      result.push({ ...seg })
    }
    i++
  }

  return result
}

/**
 * 差异片段组（用于 UI 显示）
 * 将相邻的 delete 和 insert 配对成一组
 */
export interface DiffGroup {
  id: number
  type: 'equal' | 'change'
  original: string  // 原文（delete 的内容）
  corrected: string // 新文（insert 的内容）
  useNew: boolean   // 是否使用新文本
}

/**
 * 将差异片段转换为差异组（便于 UI 渲染）
 */
export function segmentsToDiffGroups(segments: DiffSegment[]): DiffGroup[] {
  const groups: DiffGroup[] = []
  let id = 0
  let i = 0

  while (i < segments.length) {
    const seg = segments[i]

    if (seg.type === 'equal') {
      groups.push({
        id: id++,
        type: 'equal',
        original: seg.text,
        corrected: seg.text,
        useNew: false
      })
      i++
    } else {
      // 收集连续的 delete 和 insert
      let original = ''
      let corrected = ''

      while (i < segments.length && segments[i].type !== 'equal') {
        if (segments[i].type === 'delete') {
          original += segments[i].text
        } else {
          corrected += segments[i].text
        }
        i++
      }

      groups.push({
        id: id++,
        type: 'change',
        original,
        corrected,
        useNew: true // 默认使用校正后的文本
      })
    }
  }

  return groups
}

/**
 * 根据用户选择生成最终文本
 */
export function buildFinalText(groups: DiffGroup[]): string {
  return groups.map(g => {
    if (g.type === 'equal') {
      return g.original
    }
    return g.useNew ? g.corrected : g.original
  }).join('')
}
