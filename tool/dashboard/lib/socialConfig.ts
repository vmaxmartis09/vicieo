/**
 * Social Account Connection Configuration & Types
 * Handles platform-specific connection mechanisms (OAuth vs API Key / Tokens / Page Selection)
 */

export type SocialPlatformId = "tiktok" | "youtube" | "facebook" | "instagram";

export interface SocialAccountField {
  key: string;
  label: string;
  type: "text" | "password" | "select";
  placeholder?: string;
  description?: string;
  required?: boolean;
  options?: { label: string; value: string }[];
}

export interface PlatformConnectionSpec {
  id: SocialPlatformId;
  name: string;
  subtitle: string;
  brandColor: string;
  methodLabel: string;
  authType: "oauth_or_token" | "token_and_page" | "oauth_and_secret";
  description: string;
  helpUrl: string;
  fields: SocialAccountField[];
  defaultAccountName: string;
}

export interface ConnectedAccountData {
  platform: SocialPlatformId;
  connected: boolean;
  accountName?: string;
  accountId?: string;
  connectedAt?: string;
  credentials?: Record<string, string>;
}

export const SOCIAL_PLATFORMS_SPEC: Record<SocialPlatformId, PlatformConnectionSpec> = {
  tiktok: {
    id: "tiktok",
    name: "TikTok",
    subtitle: "TikTok Login Kit & Video Kit",
    brandColor: "#000000",
    methodLabel: "TikTok Creator Account",
    authType: "oauth_or_token",
    description:
      "Cần cấp quyền đăng video trực tiếp (video.upload, video.publish) thông qua TikTok for Developers hoặc Access Token của tài khoản Creator.",
    helpUrl: "https://developers.tiktok.com/",
    defaultAccountName: "@socnhotv_official",
    fields: [
      {
        key: "creator_handle",
        label: "TikTok Handle (Tên kênh)",
        type: "text",
        placeholder: "@socnhotv_official",
        required: true,
      },
      {
        key: "client_key",
        label: "Client Key / App ID",
        type: "text",
        placeholder: "aw...xyz",
        required: true,
      },
      {
        key: "access_token",
        label: "Creator Access Token",
        type: "password",
        placeholder: "act.tiktok.oauth...",
        description: "Token cấp quyền xuất bản video vào tài khoản TikTok.",
        required: true,
      },
    ],
  },
  youtube: {
    id: "youtube",
    name: "YouTube",
    subtitle: "Google Cloud / YouTube Data API v3",
    brandColor: "#FF0000",
    methodLabel: "YouTube Channel / OAuth 2.0",
    authType: "oauth_and_secret",
    description:
      "Sử dụng Google OAuth 2.0 hoặc API Credentials (Client Secret + Refresh Token) với phạm vi youtube.upload để tự động hóa đăng Shorts & Videos.",
    helpUrl: "https://console.cloud.google.com/apis/credentials",
    defaultAccountName: "Sóc Nhỏ TV Official",
    fields: [
      {
        key: "channel_title",
        label: "Tên kênh YouTube",
        type: "text",
        placeholder: "Sóc Nhỏ TV Official",
        required: true,
      },
      {
        key: "client_id",
        label: "Google Client ID",
        type: "text",
        placeholder: "xxxx.apps.googleusercontent.com",
        required: true,
      },
      {
        key: "client_secret",
        label: "Client Secret",
        type: "password",
        placeholder: "GOCSPX-...",
        required: true,
      },
      {
        key: "refresh_token",
        label: "Refresh Token",
        type: "password",
        placeholder: "1//04...",
        description: "Refresh Token giúp duy trì kết nối xuất bản mà không bị hết hạn phiên.",
        required: true,
      },
    ],
  },
  facebook: {
    id: "facebook",
    name: "Facebook",
    subtitle: "Meta Graph API (Fanpage Video)",
    brandColor: "#1877F2",
    methodLabel: "Facebook Page Video API",
    authType: "token_and_page",
    description:
      "Meta yêu cầu liên kết với một Trang (Facebook Page). Cần Page ID và Page Access Token vĩnh viễn có quyền pages_manage_posts & publish_video.",
    helpUrl: "https://developers.facebook.com/tools/explorer/",
    defaultAccountName: "Fanpage Sóc Nhỏ TV",
    fields: [
      {
        key: "page_name",
        label: "Tên Fanpage",
        type: "text",
        placeholder: "Sóc Nhỏ TV Studio",
        required: true,
      },
      {
        key: "page_id",
        label: "Facebook Page ID",
        type: "text",
        placeholder: "108392019482910",
        required: true,
      },
      {
        key: "page_access_token",
        label: "Page Access Token (Never Expire)",
        type: "password",
        placeholder: "EAA...",
        description: "Token của Fanpage lấy từ Meta Graph API Explorer.",
        required: true,
      },
    ],
  },
  instagram: {
    id: "instagram",
    name: "Instagram",
    subtitle: "Instagram Graph API / Reels Publishing",
    brandColor: "#E1306C",
    methodLabel: "Instagram Professional Account",
    authType: "token_and_page",
    description:
      "Yêu cầu tài khoản Instagram Business hoặc Creator liên kết với Fanpage Meta, sử dụng Instagram Business Account ID để xuất bản Reels tự động.",
    helpUrl: "https://developers.facebook.com/docs/instagram-api",
    defaultAccountName: "@socnhotv.reels",
    fields: [
      {
        key: "ig_username",
        label: "Tài khoản Instagram (@username)",
        type: "text",
        placeholder: "@socnhotv.reels",
        required: true,
      },
      {
        key: "ig_business_id",
        label: "Instagram Business Account ID",
        type: "text",
        placeholder: "17841400...",
        required: true,
      },
      {
        key: "access_token",
        label: "Meta User / Page Token",
        type: "password",
        placeholder: "EAA...",
        description: "Token Meta có quyền instagram_basic và instagram_content_publish.",
        required: true,
      },
    ],
  },
};

const STORAGE_KEY = "socnho_social_connections_v1";

/**
 * Lấy danh sách tài khoản đã kết nối từ Storage
 */
export function getSavedSocialAccounts(): Record<string, ConnectedAccountData> {
  if (typeof window === "undefined") {
    return {};
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    return JSON.parse(raw);
  } catch (e) {
    console.error("Failed to read social connections from localStorage:", e);
    return {};
  }
}

/**
 * Kiểm tra xem một nền tảng có đang được kết nối hợp lệ hay không
 */
export function isSocialPlatformConnected(platform: string): boolean {
  const norm = platform.toLowerCase() as SocialPlatformId;
  const accounts = getSavedSocialAccounts();
  return Boolean(accounts[norm]?.connected);
}

/**
 * Lưu kết nối mới
 */
export function saveSocialAccountConnection(
  platform: SocialPlatformId,
  accountName: string,
  credentials: Record<string, string>,
  accountId?: string
): void {
  if (typeof window === "undefined") return;
  const accounts = getSavedSocialAccounts();
  accounts[platform] = {
    platform,
    connected: true,
    accountName: accountName || SOCIAL_PLATFORMS_SPEC[platform]?.defaultAccountName,
    accountId,
    connectedAt: new Date().toISOString(),
    credentials,
  };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(accounts));
  // Dispatch custom event to notify listeners
  window.dispatchEvent(new CustomEvent("social-accounts-updated", { detail: { platform } }));
}

/**
 * Ngắt kết nối một nền tảng
 */
export function removeSocialAccountConnection(platform: SocialPlatformId): void {
  if (typeof window === "undefined") return;
  const accounts = getSavedSocialAccounts();
  if (accounts[platform]) {
    delete accounts[platform];
    localStorage.setItem(STORAGE_KEY, JSON.stringify(accounts));
    window.dispatchEvent(new CustomEvent("social-accounts-updated", { detail: { platform } }));
  }
}
