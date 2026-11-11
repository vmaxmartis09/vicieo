import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import viMessages from "@/messages/vi.json";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import { ThemeProvider } from "@/components/theme/ThemeProvider";

const mockRouterReplace = vi.fn();

vi.mock("@/i18n/routing", () => ({
  usePathname: () => "/studio",
  useRouter: () => ({ replace: mockRouterReplace }),
}));

import { UserBubble } from "@/components/navigation/UserBubble";
import { BubbleErrorBoundary } from "@/components/navigation/user-bubble/components/BubbleErrorBoundary";
import { formatStorageKb, getInitials } from "@/components/navigation/user-bubble/utils/storageUtils";

let savedKeyPayloads: Array<{ provider: string; apiKey: string }> = [];

const server = setupServer(
  http.get(`${CORE_URL}/health`, () =>
    HttpResponse.json({ status: "healthy", version: "1.2.3", repo_root: "/Volumes/Data/socnhotv" })
  ),
  http.get(`${CORE_URL}/video/templates`, () => HttpResponse.json([])),
  http.get(`${CORE_URL}/settings/api-keys`, () =>
    HttpResponse.json({
      providers: [{ provider: "gemini", configured: true, updatedAt: "2026-11-09T00:00:00Z" }],
    })
  ),
  http.post(`${CORE_URL}/settings/api-keys`, async ({ request }) => {
    const body = (await request.json()) as { provider: string; apiKey: string };
    savedKeyPayloads.push(body);
    return HttpResponse.json({ ok: true });
  })
);

function renderBubble(props: React.ComponentProps<typeof UserBubble> = {}) {
  return render(
    <NextIntlClientProvider locale="vi" messages={viMessages}>
      <ThemeProvider>
        <UserBubble {...props} />
      </ThemeProvider>
    </NextIntlClientProvider>
  );
}

beforeAll(() => server.listen());
beforeEach(() => {
  savedKeyPayloads = [];
  mockRouterReplace.mockReset();
  useStudioStore.setState({
    theme: "light",
    aspectRatio: "9:16",
    isMockMode: true,
    userProfile: { name: "Đạo diễn Sóc Nhỏ", email: "director@socnhotv.vn" },
    apiKeyStatus: {
      gemini: { configured: false, updatedAt: null },
      fal: { configured: false, updatedAt: null },
      openai: { configured: false, updatedAt: null },
      replicate: { configured: false, updatedAt: null },
    },
    coreHealth: { online: false, version: "–" },
  });
});
afterEach(() => {
  server.resetHandlers();
  cleanup();
});
afterAll(() => server.close());

describe("UserBubble characterization tests", () => {
  it("opens settings dialog, fetches core health + keys, toggles sections, saves API key, and handles 2-step clear confirmation", async () => {
    const user = userEvent.setup();
    renderBubble();

    const trigger = screen.getByRole("button", { name: "Tài khoản & Cài đặt Studio" });
    expect(trigger).toHaveAttribute("aria-expanded", "false");

    await user.click(trigger);

    const dialog = await screen.findByRole("dialog", { name: "Hồ sơ & Cài đặt Studio" });
    expect(dialog).toBeInTheDocument();

    // Synced core health + 1 configured API key
    await waitFor(() => {
      expect(screen.getByText("Core v1.2.3")).toBeInTheDocument();
      expect(screen.getByText("1/4")).toBeInTheDocument();
    });

    // Switch language
    await user.click(screen.getByRole("button", { name: /Ngôn ngữ/i }));
    await user.click(screen.getByRole("button", { name: "English" }));
    expect(mockRouterReplace).toHaveBeenCalledWith("/studio", { locale: "en" });

    // Switch theme
    await user.click(screen.getByRole("button", { name: /Giao diện/i }));
    await user.click(screen.getByRole("button", { name: "Tối" }));
    expect(useStudioStore.getState().theme).toBe("dark");

    // Configure an API key
    await user.click(screen.getByRole("button", { name: /Khóa API nhà cung cấp/i }));
    await user.click(screen.getByRole("button", { name: "Nhập khóa: OpenAI" }));
    const keyInput = screen.getByPlaceholderText("Dán khóa API vào đây...");
    await user.type(keyInput, "sk-openai-secret");
    await user.click(screen.getByRole("button", { name: "Lưu khóa" }));

    await waitFor(() => {
      expect(savedKeyPayloads).toEqual([{ provider: "openai", apiKey: "sk-openai-secret" }]);
      expect(screen.getByText("2/4")).toBeInTheDocument();
    });

    // 2-step local storage clear prompt & cancel
    await user.click(screen.getByRole("button", { name: /Nơi lưu dữ liệu/i }));
    await user.click(screen.getByRole("button", { name: "Xóa dữ liệu cục bộ" }));
    expect(screen.getByText("Bước 1/2: Bạn có chắc muốn xóa dữ liệu cục bộ?")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Xác nhận" }));
    expect(screen.getByText("Bước 2/2: Xác nhận xóa vĩnh viễn bộ nhớ trình duyệt")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Giữ lại dữ liệu" }));
    expect(screen.getByRole("button", { name: "Xóa dữ liệu cục bộ" })).toBeInTheDocument();
  });

  it("supports 'beside' placement inline profile drawer and closes on Escape", async () => {
    const user = userEvent.setup();
    renderBubble({ compact: true, placement: "beside" });

    await user.click(screen.getByRole("button", { name: "Tài khoản & Cài đặt Studio" }));
    const profileBtn = await screen.findByRole("button", { name: "Sửa hồ sơ" });
    await user.click(profileBtn);

    const nameInput = screen.getByRole("textbox", { name: "Hồ sơ & Cài đặt Studio" }) as HTMLInputElement;
    expect(nameInput.value).toBe("Đạo diễn Sóc Nhỏ");
    await user.clear(nameInput);
    await user.type(nameInput, "Bi Director");
    expect(useStudioStore.getState().userProfile.name).toBe("Bi Director");

    await user.keyboard("{Escape}");
    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: "Hồ sơ & Cài đặt Studio" })).not.toBeInTheDocument();
    });
  });

  it("computes initials/storage strings and isolates BubbleErrorBoundary crash", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const user = userEvent.setup();
    expect(getInitials("Sóc Nhỏ TV")).toBe("SN");
    expect(formatStorageKb(2048)).toBe("2.0 KB");

    let fail = true;
    function CrashChild() {
      if (fail) throw new Error("Boom");
      return <span>Bubble OK</span>;
    }

    render(
      <BubbleErrorBoundary>
        <CrashChild />
      </BubbleErrorBoundary>
    );
    const retryBtn = screen.getByTitle("Lỗi menu tài khoản — bấm thử lại");
    expect(retryBtn).toBeInTheDocument();
    fail = false;
    await user.click(retryBtn);
    expect(screen.getByText("Bubble OK")).toBeInTheDocument();
    consoleSpy.mockRestore();
  });
});
