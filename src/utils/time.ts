import type { TimeStamp } from '@/types/subtitle'

/**
 * 将 TimeStamp 转换为毫秒数
 */
export function timeStampToMs(time: TimeStamp): number {
  return (
    time.hours * 3600000 + time.minutes * 60000 + time.seconds * 1000 + time.milliseconds
  )
}

/**
 * 将毫秒数转换为 TimeStamp
 */
export function msToTimeStamp(ms: number): TimeStamp {
  // 确保输入是整数，避免浮点数导致 Rust 后端解析失败
  ms = Math.round(ms)
  const hours = Math.floor(ms / 3600000)
  ms = ms % 3600000
  const minutes = Math.floor(ms / 60000)
  ms = ms % 60000
  const seconds = Math.floor(ms / 1000)
  const milliseconds = ms % 1000

  return { hours, minutes, seconds, milliseconds }
}

/**
 * 将 TimeStamp 转换为 SRT 格式字符串 (HH:MM:SS,mmm)
 */
export function timeStampToString(time: TimeStamp): string {
  const h = String(time.hours).padStart(2, '0')
  const m = String(time.minutes).padStart(2, '0')
  const s = String(time.seconds).padStart(2, '0')
  const ms = String(time.milliseconds).padStart(3, '0')
  return `${h}:${m}:${s},${ms}`
}

/**
 * 将 SRT 格式时间字符串解析为 TimeStamp
 */
export function parseTimeString(timeStr: string): TimeStamp {
  const match = timeStr.match(/(\d{2}):(\d{2}):(\d{2}),(\d{3})/)
  if (!match || !match[1] || !match[2] || !match[3] || !match[4]) {
    throw new Error(`Invalid time format: ${timeStr}`)
  }

  return {
    hours: parseInt(match[1], 10),
    minutes: parseInt(match[2], 10),
    seconds: parseInt(match[3], 10),
    milliseconds: parseInt(match[4], 10),
  }
}

/**
 * 格式化时间为可读字符串 (MM:SS)
 */
export function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

/**
 * 计算两个时间戳之间的持续时间(毫秒)
 */
export function getDuration(start: TimeStamp, end: TimeStamp): number {
  return timeStampToMs(end) - timeStampToMs(start)
}
