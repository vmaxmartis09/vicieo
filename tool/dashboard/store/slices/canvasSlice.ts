/**
 * @file Canvas Slice Re-export Hub
 * Toàn bộ logic đã được module hoá vào `store/slices/canvas/*` theo đúng kiến trúc ai/plans/refactor-components.md.
 * File này đóng vai trò re-export giữ nguyên tương thích 100% cho tất cả nơi đang import từ @/store/slices/canvasSlice.
 */
export * from "./canvas/index";
