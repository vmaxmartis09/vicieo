# SOCNHOTV AGENT OPERATING MANIFESTO: RELENTLESS EXECUTION BY DEFAULT
# (QUY TẮC HOẠT ĐỘNG BẮT BUỘC: MẶC ĐỊNH LÀM VIỆC ĐẾN CÙNG - KHÔNG CẦN GÕ /GOAL)

> [!CRITICAL]
> **ĐIỀU KHOẢN TỐI CAO: CHẾ ĐỘ "GOAL-DRIVEN" LÀ MẶC ĐỊNH TRONG MỌI TÌNH HUỐNG**
> - Người dùng **KHÔNG CẦN** và **KHÔNG BAO GIỜ PHẢI GÕ `/goal`**.
> - Agent **BẮT BUỘC MẶC ĐỊNH** kích hoạt tinh thần và quy trình `/goal` (kiên trì cùng cực, tự động thích ứng, đào sâu giải quyết đến cùng) cho **MỌI YÊU CẦU BÌNH THƯỜNG**.
> - Tuyệt đối không bao giờ dừng lại khi gặp lỗi đầu tiên, không hỏi lại những điều có thể tự suy luận hoặc tự debug, không đẩy việc lại cho người dùng khi chưa cạn kiệt mọi giải pháp.

---

## 1. NGUYÊN TẮC CỐT LÕI: TUYỆT ĐỐI KHÔNG BỎ CUỘC (NEVER SURRENDER PROTOCOL)
- **Tư duy đến cùng (Tenacity First):** Agent KHÔNG BAO GIỜ được phép bỏ cuộc, dừng lại nửa chừng, than phiền "tôi không làm được", hoặc báo thất bại khi chưa thử nghiệm tối thiểu 3 - 5 chiến lược độc lập.
- **Sự cố là dữ liệu phản hồi, không phải điểm dừng:** Mọi trở ngại (DOM thay đổi, selector không match, network timeout, Google OAuth 2FA/approval, Captcha, anti-bot detection...) là tín hiệu để lập tức chuyển sang chiến lược dự phòng kế tiếp.
- **Vòng lặp chuyển đổi chiến lược bắt buộc (Multi-Strategy Pivot Loop):**
  1. *Chiến lược 1 (Direct Selector):* Thử selector đa ngôn ngữ (EN/VI), ARIA attributes, semantic roles, data attributes (`data-e2e`).
  2. *Chiến lược 2 (DOM Traversal & Text Match):* Dùng XPath tương đối, tìm thẻ cha/con, duyệt text content qua JavaScript `page.evaluate()`.
  3. *Chiến lược 3 (Input & Mouse Simulation):* Dùng tọa độ chuột (bounding box click), bàn phím ảo, giả lập thao tác tự nhiên.
  4. *Chiến lược 4 (Browser Environment Pivot):* Nếu chạy Headless bị phát hiện hoặc chặn bot -> Lập tức chuyển sang Headed mode (mở trình duyệt thật trên màn hình).
  5. *Chiến lược 5 (CDP & Session/Cookie Injection):* Can thiệp qua Chrome DevTools Protocol (CDP), nạp profile độc lập, tái sử dụng session/cookies đã xác thực.

---

## 2. QUY CHUẨN TỰ ĐỘNG HÓA TRÌNH DUYỆT (BROWSER AUTOMATION STANDARDS)
Khi thực hiện các tác vụ tự động hóa trình duyệt (đặc biệt là tạo tài khoản, đăng nhập mạng xã hội như TikTok, Google, Facebook, eCommerce):

### 2.1. Môi trường Trình duyệt: Ego Lite (Mặc định)
- **Chuẩn hóa trên Ego Lite:** Luôn sử dụng trình duyệt **Ego Lite** (`ego-browser` CLI, Node runtime và skill `ego-browser`).
- **Tận dụng Session đăng nhập sẵn có:** Ego Lite chia sẻ trực tiếp profile và tài khoản đã đăng nhập của người dùng, không cần tạo profile giả lập.
- **Thực thi qua Ego Task Spaces:** Khởi tạo không gian tác vụ qua `taskSpace()`, thao tác page với human-like actions, và gọi `task.finish({ keep: [] })` khi hoàn tất.

### 2.2. Vượt cơ chế phòng thủ Anti-Bot (TikTok, Google, Cloudflare)
- **Cờ khởi chạy tàng hình (Stealth Flags):** Luôn kèm theo:
  - `--disable-blink-features=AutomationControlled`
  - `--no-first-run`
  - `--no-default-browser-check`
- **Chế độ hiển thị (Headed vs Headless):**
  - Với các nền tảng có hệ thống kiểm tra bot khắt khe như TikTok/Google: **BẮT BUỘC ưu tiên Headed mode** hoặc mở trực tiếp trên profile thực tế để tránh bị thuật toán chặn ngay từ đầu.
- **Hành vi tự nhiên (Human-like Behavior):**
  - Thêm khoảng nghỉ ngẫu nhiên (jitter delay 500ms - 2000ms) giữa các lần click, nhập liệu.
  - Tránh click dồn dập hoặc gửi phím siêu tốc khiến hệ thống nghi ngờ.

### 2.3. Xử lý Đa Tab & Cửa sổ Popup OAuth
- Không bao giờ giả định trình duyệt chỉ có 1 page.
- Khi bấm nút OAuth ("Continue with Google", "Login with Facebook"...):
  - Luôn chuẩn bị bắt popup qua `context.expect_page()` hoặc lắng nghe `context.on("page")`.
  - Tự động chuyển ngữ cảnh sang cửa sổ popup để chọn tài khoản, bấm "Tiếp tục / Continue / Cho phép".
  - Chờ popup tự đóng hoặc hoàn tất xác thực, sau đó quay lại tab chính để kiểm tra.

### 2.4. Đối phó CAPTCHA & Mã xác nhận (OTP / 2FA)
- Nếu xuất hiện CAPTCHA dạng trượt hoặc chọn hình:
  - Tự động chụp ảnh lưu vào `shots/` của project hiện hành và cập nhật thông báo.
  - Nếu cần người dùng hỗ trợ giải CAPTCHA hoặc nhập OTP cá nhân: **GIỮ NGUYÊN TRÌNH DUYỆT MỞ TRÊN MÀN HÌNH**, hướng dẫn rõ ràng cho người dùng, chủ động theo dõi DOM để ngay khi người dùng hoàn thành, Agent tự động tiếp quản và thực hiện tiếp 100% các bước còn lại. **TUYỆT ĐỐI KHÔNG ĐÓNG TRÌNH DUYỆT HAY HỦY TÁC VỤ**.

---

## 3. QUY CHUẨN KIẾN TRÚC 3 PHẦN & ĐA-PROJECT (ZERO ROOT POLLUTION)
Hệ thống phân chia ranh giới triệt để thành 3 phần độc lập:
1. **`ai/`**: Nơi lưu trữ trí tuệ cho các Agent AI (Claude, Cursor, Codex, Antigravity, Gemini). Chứa:
   - `ai/AGENTS.md` (Nguồn sự thật duy nhất về luật chơi).
   - `ai/skills/`, `ai/rules/`, `ai/workflows/`.
   - `ai/map/` (`TOOLS.md`, `DATA.md` để AI định vị khả năng và dữ liệu).
   - `ai/memory/` (`CONTINUITY.md` ghi nhớ trạng thái phiên làm việc).
2. **`tool/`**: Phần mềm và engine duy nhất dùng chung cho mọi project:
   - Code logic tại `tool/src/socnhotv/` (`core/`, `workspace/`, `video/`, `social/`, `intelligence/`, `adapters/`, `cli/`).
   - Web Dashboard tại `tool/dashboard/` (hỗ trợ chuyển đổi project).
     - Kiểm production build bằng `NEXT_DIST_DIR=.next-build npx next build`.
     - KHÔNG BAO GIỜ build vào `.next` khi dev server cổng 3000 đang chạy.
   - Tuyệt đối không chứa dữ liệu cá nhân hay dữ liệu người dùng trong `tool/`.
   - Mọi đường dẫn dữ liệu đi qua module `workspace/` (resolve path theo project, `--project <id>`).
3. **`data/`**: Toàn bộ dữ liệu độc lập:
   - `data/workspace.json`: Khai báo danh sách project, project active mặc định (`soc-nho-tv`).
   - `data/shared/`: Dữ liệu dùng chung (`voice_vault/`, `models/`, `library/`, `brand_kits/`, `templates/`, `music_sfx/`, `knowledge/`).
   - `data/projects/<project-id>/`: Dữ liệu riêng biệt từng project (`characters/`, `library/`, `products/`, `accounts/`, `content/`, `publishing/`, `analytics/`, `knowledge/`, `raw/`, `output/`, `shots/`, `data.manifest.json`).
4. **Chống phình to dung lượng (Anti-Bloat Protocol):** Bắt buộc tuân thủ `REPO_OPTIMIZATION_GUIDE.md`:
   - Dùng duy nhất một `.venv` ở thư mục gốc, cấm tạo sub-venv trong `tool/toolbox/tools/`.
   - Luôn dọn sạch `git worktree remove` và `git worktree prune` khi Agent kết thúc session (không để sót worktrees trong `.claude/worktrees`).
   - Xóa ngay cache `.next-build` sau khi kiểm tra build dashboard.

5. **Khởi chạy Hệ thống đa nền tảng (Dual-Run Cross-Platform):**
   - Hỗ trợ chạy đồng thời cả Backend Rust (`engine-rust`, port 9000) và Frontend Next.js (`tool/dashboard`, port 3600) bằng 1 lệnh DUY NHẤT:
     ```bash
     vcrun
     ```

---

## 4–6, 8. TRI THỨC DỰ ÁN (nhân vật, âm thanh, HyperFrames, tự học, video craft)
- Đã chuyển sang `ai/knowledge/PROJECT_VIDEO_KNOWLEDGE.md`. Đọc khi làm kịch bản/dựng video, không nạp mỗi lượt.

---

## 7. QUY ƯỚC FRONTEND
- Icon: chỉ dùng Morphicons, cấm import `lucide-react`. Chi tiết: `ai/rules/morphicons.md`. Quy ước FE khác: `ai/rules/ui-rules.md`.
