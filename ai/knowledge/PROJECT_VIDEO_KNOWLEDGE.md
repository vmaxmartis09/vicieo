# TRI THỨC DỰ ÁN: NHÂN VẬT, ÂM THANH, DỰNG VIDEO (project knowledge, không phải chỉ thị cho Claude)

## 4. QUY CHUẨN NHÂN VẬT & ÂM THANH
- **Nhân vật Sóc Nhỏ (Bi) & Bạn Koala:**
  - Nhân vật chính dùng master DNA ảnh trong `characters/soc_nho/`, tuyệt đối không tự ý vẽ nhân vật mới làm lệch nhận diện.
  - Tỉ lệ nhân vật khóa cứng: **Sóc Nhỏ (Bi) 100cm, Koala 150cm** (tuân thủ `SCALE_LOCK.md`).
- **Voiceover Sóc Nhỏ:**
  - Luôn sử dụng voice-id `soc-nho` trong `data/shared/voice_vault/soc-nho/` (OmniVoice SOTA).
  - Mọi audio thoại đi qua chuỗi TikTok Mastering chuẩn: de-esser, EQ presence +3.5dB @3.2kHz, compressor 4:1, -14 LUFS, MP3 192kbps 44.1kHz.

## 5. QUY CHUẨN CÔNG CỤ DỰNG & BIÊN TẬP VIDEO: HEYGEN HYPERFRAMES (MẶC ĐỊNH BẮT BUỘC)
- **Công cụ mặc định duy nhất:** Toàn bộ video KOC AI, hoạt hình 3D, social clips được biên tập bằng composition HTML5/CSS/GSAP của **HeyGen HyperFrames** (`ai/skills/hyperframes*`, `tool/src/socnhotv/video/hyperframes_engine.py`).
- **Loại bỏ công cụ cũ:** Không dùng ghép video FFmpeg filtergraph thủ công cứng nhắc hoặc CapCut khi hệ thống có thể tự động hóa bằng HyperFrames.
- **Tiêu chuẩn dựng video với HyperFrames:**
  1. *Cấu trúc HTML5 tất định (Deterministic):* Tận dụng HTML, CSS3, GSAP seekable timeline.
  2. *Chuẩn TikTok Safe Zone 9:16 (1080x1920):* Luôn giữ vùng an toàn cho top header (180px) và bottom cart/controls (320px).
  3. *Đồng bộ đa luồng âm thanh:* Voiceover chuẩn nét kết hợp BGM ducking tự động.
  4. *Lệnh thực thi chuẩn:* Chạy qua CLI `python -m socnhotv --project <id> video render` hoặc gọi trực tiếp `HyperFramesEngine`.

## 6. GIAO THỨC TỰ HỌC & ĐÚC KẾT KINH NGHIỆM
- Sau mỗi đợt render hoặc chiến dịch, Agent ghi nhận chỉ số thực tế (retention, views, tương tác) vào `ai/memory/CONTINUITY.md` và `data/projects/<project-id>/analytics/`.
- Tự động rút kinh nghiệm tối ưu hóa kịch bản và prompt cho các tập tiếp theo mà không lặp lại lỗi cũ.

## 8. QUY CHUẨN TRI THỨC VIDEO THỰC CHIẾN (VIDEO CRAFT KNOWLEDGE)
- **Tra cứu bắt buộc:** Trước khi viết kịch bản, sinh prompt hình ảnh/b-roll, dựng composition HyperFrames hoặc xuất bản video, Agent **BẮT BUỘC** đọc skill `ai/skills/video-craft` và tra cứu `data/shared/knowledge/video_craft/techniques/STATUS.md` (hoặc `registry.json`).
- **Ghi nhận kỹ thuật mới:** Mọi kỹ thuật mới học được phải ghi thành thẻ markdown trong `techniques/` có đầy đủ YAML front-matter chuẩn (`status`, `reproducible_with_our_stack`, `needs`, `confidence`, `last_verified`, `expires_at`).
- **Tận dụng các module craft sẵn có:** Ưu tiên sử dụng các module trong `tool/src/socnhotv/video/craft/` (`hooks`, `depth`, `transitions`, `camera`, `lint`) thay vì viết lại logic thủ công.
